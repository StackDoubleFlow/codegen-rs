use crate::data::*;
use crate::helpers::{create_ident, create_ident_trimmed};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use quote::quote;
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
}

impl Field {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let name = create_ident_trimmed(&self.name);
        let type_ref = self.field_type.write_qualified_name(types);
        quote! {
            #name: #type_ref
        }
    }
}

impl Method {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let name = create_ident(&self.name);
        let return_type = self.return_type.write_qualified_name(types);
        quote! {
            pub fn #name() -> #return_type {

            }
        }
    }
}

impl TypeData {
    fn write_deref(&self, name: &Ident, types: &DllData) -> Option<TokenStream> {
        let parent = self.parent.as_ref()?;
        let super_type = parent.write_qualified_name(types);
        Some(quote! {
            impl std::ops::Deref for #name {
                type Target = #super_type;

                fn deref(&self) -> &Self::Target {
                    &self.super_
                }
            }
        })
    }

    fn write_class(&self, types: &DllData) -> TokenStream {
        let name = if let Some(nested_parent) = &self.this.declaring_type {
            let name = nested_parent.full_name(types) + "_" + &self.this.name;
            create_ident(&name)
        } else {
            create_ident(&self.this.name)
        };
        let fields = self.instance_fields.iter().map(|f| f.write_tokens(types));
        let super_field = self.parent.as_ref().map(|parent| {
            let super_ident = create_ident("super_");
            let super_type = parent.write_qualified_name(types);
            quote! {
                #super_ident: #super_type,
            }
        });
        let methods = self.methods.iter().map(|m| m.write_tokens(types));
        let deref = self.write_deref(&name, types);

        quote! {
            #[repr(C)]
            pub struct #name {
                #super_field
                #( pub #fields ),*
            }

            impl #name {
                #( #methods )*
            }

            #deref
        }
    }

    fn write_tokens(&self, types: &DllData) -> TokenStream {
        match self.type_enum {
            TypeEnum::Class | TypeEnum::Struct => self.write_class(types),
            _ => quote! {},
        }
    }
}

#[derive(Default)]
struct Module<'a> {
    children: HashMap<String, Module<'a>>,
    types: Vec<&'a TypeData>
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

        global_module.write_tokens(self)
    }
}

impl<'a> Module<'a> {
    fn write_tokens(&self, types: &DllData) -> TokenStream {
        let children_names = self.children.keys().map(|s| create_ident(s));
        let children = self.children.values().map(|module| module.write_tokens(types));
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