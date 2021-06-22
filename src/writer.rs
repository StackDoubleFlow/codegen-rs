use crate::data::*;
use crate::helpers::{create_ident, create_ident_trimmed};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use std::lazy::SyncOnceCell;

macro_rules! replacement_types {
    ( $( $name:ident => $replacement:ty ),* ) => {
        #[derive(Debug)]
        struct ReplacementTypes {
            $(
                $name: i32
            ),*
        }

        impl ReplacementTypes {
            fn replace(&self, id: i32) -> Option<TokenStream> {
                return
                $(
                    if id == self.$name {
                        Some(quote! { $replacement })
                    } else
                )*
                { None }
            }
        }
    };
}

replacement_types! {
    single => f32,
    double => f64,
    void => (),
    int16 => i16,
    int32 => i32,
    int64 => i64,
    uint16 => u16,
    uint32 => u32,
    uint64 => u64,
    byte => u8,
    sbyte => i8,
    boolean => bool,
    object => quest_hook::libil2cpp::Il2CppObject,
    string => quest_hook::libil2cpp::Il2CppString
}

static REPLACEMENT_TYPES: SyncOnceCell<ReplacementTypes> = SyncOnceCell::new();

impl TypeRef {
    fn full_name(&self, types: &DllData) -> String {
        let mut name = self.name.clone();

        let mut current = &types[self];
        while let Some(parent) = &current.this.declaring_type {
            name.insert(0, '_');
            name.insert_str(0, &parent.name);
            current = &types[parent];
        }

        name
    }

    fn get_qualified_name(&self, types: &DllData) -> TokenStream {
        let mut name = self.name.clone();
        let mut namespace = &self.namespace;

        let mut current = &types[self];
        while let Some(parent) = &current.this.declaring_type {
            name.insert(0, '_');
            name.insert_str(0, &parent.name);
            namespace = &parent.namespace;
            current = &types[parent];
        }

        let namespace_tokens = namespace.split_terminator('.').map(create_ident);
        let name_ident = create_ident(&name);
        let generics = if !self.generics.is_empty() {
            let args = self.generics.iter().map(|tr| tr.write_instance_type(types));
            Some(quote! { < #( #args ),* > })
        } else {
            None
        };

        quote! {
            crate:: #( #namespace_tokens :: )* #name_ident #generics
        }
    }

    fn write_qualified_name(&self, types: &DllData) -> TokenStream {
        if self.type_id >= 0 {
            self.get_qualified_name(types)
        } else {
            create_ident(&self.name).into_token_stream()
        }
    }

    fn write_instance_type(&self, types: &DllData) -> TokenStream {
        let prefix = if self.type_id > 0 && types[self].pass_by_ref() {
            Some(quote! { *mut })
        } else {
            None
        };
        let replacements = REPLACEMENT_TYPES.get().unwrap();
        let name = if let Some(replacement) = replacements.replace(self.type_id) {
            replacement
        } else {
            self.write_qualified_name(types)
        };
        let ty = quote! { #prefix #name };
        if self.is_array {
            quote! { *mut quest_hook::libil2cpp::Il2CppArray< #ty > }
        } else {
            ty
        }
    }
}

impl Field {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let name = create_ident_trimmed(&self.name);
        let type_ref = self.field_type.write_instance_type(types);
        quote! {
            #name: #type_ref
        }
    }
}

impl Method {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let name_str = &self.name;
        let name = create_ident(&self.name);
        let param_names = self.parameters.iter().enumerate().map(|(i, p)| {
            if p.name.is_empty() {
                create_ident(&format!("_param{}", i))
            } else {
                create_ident(&p.name)
            }
        });
        let param_names2 = param_names.clone();
        let param_types = self
            .parameters
            .iter()
            .map(|p| p.parameter_type.write_instance_type(types));
        let return_type = self.return_type.write_instance_type(types);
        let generics = if !self.generic_parameters.is_empty() {
            let args = self
                .generic_parameters
                .iter()
                .map(|tr| create_ident(&tr.name));
            Some(quote! { < #( #args ),* > })
        } else {
            None
        };
        let doc = format!("Offset: {:0X}", self.offset);

        quote! {
            #[doc = #doc]
            pub fn #name #generics ( #( #param_names: #param_types ),* ) -> #return_type {
                Self::class().invoke(#name_str, #( #param_names2 ),* )
            }
        }
    }
}

impl TypeData {
    fn write_deref(
        &self,
        name: &Ident,
        generics: &Option<TokenStream>,
        types: &DllData,
    ) -> Option<TokenStream> {
        let parent = self.parent.as_ref()?;
        let super_type = parent.write_qualified_name(types);
        Some(quote! {
            impl #generics std::ops::Deref for #name #generics {
                type Target = #super_type;

                fn deref(&self) -> &Self::Target {
                    &self.super_
                }
            }
        })
    }

    fn full_name(&self, types: &DllData) -> Ident {
        if let Some(nested_parent) = &self.this.declaring_type {
            let name = nested_parent.full_name(types) + "_" + &self.this.name;
            create_ident(&name)
        } else {
            create_ident(&self.this.name)
        }
    }

    fn pass_by_ref(&self) -> bool {
        matches!(self.type_enum, TypeEnum::Class | TypeEnum::Interface)
    }

    fn write_class(&self, types: &DllData) -> TokenStream {
        let namespace = &self.this.namespace;
        let name_lit = &self.this.name;
        let name = self.full_name(types);
        let fields = self.instance_fields.iter().map(|f| f.write_tokens(types));
        let super_field = self.parent.as_ref().map(|parent| {
            let super_ident = create_ident("super_");
            let super_type = parent.write_qualified_name(types);
            quote! {
                #super_ident: #super_type,
            }
        });
        let methods = self.methods.iter().map(|m| m.write_tokens(types));
        let generics = if !self.this.generics.is_empty() {
            let args = self.this.generics.iter().map(|tr| create_ident(&tr.name));
            Some(quote! { < #( #args ),* > })
        } else {
            None
        };
        let deref = self.write_deref(&name, &generics, types);

        quote! {
            #[repr(C)]
            pub struct #name #generics {
                #super_field
                #( pub #fields ),*
            }

            impl #generics #name #generics {
                #( #methods )*
            }

            unsafe impl #generics quest_hook::libil2cpp::Type for #name #generics {
                const NAMESPACE: &'static str = #namespace;
                const CLASS_NAME: &'static str = #name_lit;
            }

            #deref
        }
    }

    fn write_interface(&self, types: &DllData) -> TokenStream {
        let name = self.full_name(types);
        let methods = self.methods.iter().map(|m| m.write_tokens(types));
        let generics = if !self.this.generics.is_empty() {
            let args = self.this.generics.iter().map(|tr| create_ident(&tr.name));
            Some(quote! { < #( #args ),* > })
        } else {
            None
        };

        quote! {
            pub struct #name #generics;

            impl #generics #name #generics {
                #( #methods )*
            }
        }
    }

    fn write_enum(&self, types: &DllData) -> TokenStream {
        let name = self.full_name(types);
        let variants = self.static_fields.iter().map(|f| create_ident(&f.name));
        let generics = if !self.this.generics.is_empty() {
            let args = self.this.generics.iter().map(|tr| create_ident(&tr.name));
            Some(quote! { < #( #args ),* > })
        } else {
            None
        };
        let ty = self.instance_fields[0]
            .field_type
            .write_instance_type(types);

        quote! {
            #[repr( #ty )]
            pub enum #name #generics {
                #( #variants ),*
            }
        }
    }

    fn write_tokens(&self, types: &DllData) -> TokenStream {
        match self.type_enum {
            TypeEnum::Class | TypeEnum::Struct => self.write_class(types),
            TypeEnum::Enum => self.write_enum(types),
            TypeEnum::Interface => self.write_interface(types),
        }
    }
}

#[derive(Default)]
struct Module<'a> {
    children: HashMap<String, Module<'a>>,
    types: Vec<&'a TypeData>,
}

impl DllData {
    fn find_type(&self, namespace: &str, name: &str) -> i32 {
        self.types
            .iter()
            .position(|ty| ty.this.namespace == namespace && ty.this.name == name)
            .unwrap() as i32
    }

    pub fn write_tokens(&self) -> TokenStream {
        // println!("{}", serde_json::to_string(&self.types[19]).unwrap());

        let replacements = ReplacementTypes {
            single: self.find_type("System", "Single"),
            double: self.find_type("System", "Double"),
            void: self.find_type("System", "Void"),
            int16: self.find_type("System", "Int16"),
            int32: self.find_type("System", "Int32"),
            int64: self.find_type("System", "Int64"),
            uint16: self.find_type("System", "UInt16"),
            uint32: self.find_type("System", "UInt32"),
            uint64: self.find_type("System", "UInt64"),
            byte: self.find_type("System", "Byte"),
            sbyte: self.find_type("System", "SByte"),
            boolean: self.find_type("System", "Boolean"),
            object: self.find_type("System", "Object"),
            string: self.find_type("System", "String"),
        };
        REPLACEMENT_TYPES.set(replacements).unwrap();

        let mut global_module = Module::default();
        for ty in &self.types {
            let namespace = ty.this.namespace.split_terminator('.');
            let mut module = &mut global_module;
            for part in namespace {
                module = module.children.entry(part.to_owned()).or_default();
            }
            module.types.push(ty);
        }

        let code = global_module.write_tokens(self);

        quote! {
            #![allow(warnings)]

            #code
        }
    }
}

impl<'a> Module<'a> {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let children_names = self.children.keys().map(|s| create_ident(s));
        let children = self
            .children
            .values()
            .map(|module| module.write_tokens(types));
        let types = self.types.iter().cloned().map(|td| td.write_tokens(types));
        quote! {
            #(
                pub mod #children_names {
                    #children
                }
            )*

            #(
                #types
            )*
        }
    }
}
