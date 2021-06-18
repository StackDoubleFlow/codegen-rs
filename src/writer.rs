use crate::data::*;
use crate::helpers::{create_ident, create_ident_trimmed};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;

fn get_qualified_name(namespace: &str, name: &str) -> TokenStream {
    let namespace_tokens = namespace.split_terminator('.').map(create_ident);
    let name_ident = create_ident(name);
    quote! {
        crate:: #( #namespace_tokens :: )* #name_ident
    }
}

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

    fn write_qualified_name(&self, types: &DllData) -> TokenStream {
        if self.type_id > 0 {
            let name = self.full_name(types);
            get_qualified_name(&self.namespace, &name)
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
        let name = self.write_qualified_name(types);
        quote! { #prefix #name }
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
        let name = create_ident(&self.name);
        let param_names = self.parameters.iter().enumerate().map(|(i, p)| {
            if p.name.is_empty() {
                create_ident(&format!("_param{}", i))
            } else {
                create_ident(&p.name)
            }
        });
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
        quote! {
            pub fn #name #generics ( #( #param_names: #param_types ),* ) -> #return_type {

            }
        }
    }
}

impl TypeData {
    fn write_deref(&self, name: &Ident, generics: &Option<TokenStream>, types: &DllData) -> Option<TokenStream> {
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

            impl #generics quest_hook::libil2cpp::Type for #name #generics {
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

        quote! {
            #[repr(C)]
            enum #name {
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
    pub fn write_tokens(&self) -> TokenStream {
        let mut global_module = Module::default();

        for ty in &self.types {
            let namespace = ty.this.namespace.split_terminator('.');
            let mut module = &mut global_module;
            for part in namespace {
                module = module.children.entry(part.to_owned()).or_default();
            }
            module.types.push(ty);
        }

        // println!("{}", serde_json::to_string(&self.types[19]).unwrap());

        global_module.write_tokens(self)
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
