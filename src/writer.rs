use crate::data::*;
use crate::helpers::create_ident;
use proc_macro2::{Ident, TokenStream};
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
    fn write_tokens(&self) -> TokenStream {
        get_qualified_name(&self.namespace, &self.name)
    }
}

impl Field {
    fn write_tokens(&self) -> TokenStream {
        let name = create_ident(&self.name);
        let type_ref = self.field_type.write_tokens();
        quote! {
            #name: #type_ref
        }
    }
}

impl Method {
    fn write_tokens(&self) -> TokenStream {
        let name = create_ident(&self.name);
        let return_type = self.return_type.write_tokens();
        quote! {
            pub fn #name() -> #return_type {

            }
        }
    }
}

impl TypeData {
    fn write_deref(&self, name: &Ident) -> Option<TokenStream> {
        let parent = self.parent.as_ref()?;
        let super_type = parent.write_tokens();
        Some(quote! {
            impl std::ops::Deref for #name {
                type Target = #super_type;

                fn deref(&self) -> &Self::Target {
                    &self.super_
                }
            }
        })
    }

    fn write_class(&self) -> TokenStream {
        let name = create_ident(&self.this.name);
        let fields = self.instance_fields.iter().map(Field::write_tokens);
        let super_field = self.parent.as_ref().map(|parent| {
            let super_ident = create_ident("super_");
            let super_type = parent.write_tokens();
            quote! {
                #super_ident: #super_type,
            }
        });
        let deref = self.write_deref(&name);
        let methods = self.methods.iter().map(Method::write_tokens);
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

    fn write_tokens(&self) -> TokenStream {
        match self.type_enum {
            // TypeEnum::Enum => quote! {
            //     enum #name {

            //     }
            // },
            TypeEnum::Class | TypeEnum::Struct => self.write_class(),
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

        global_module.write_tokens()
    }
}

impl<'a> Module<'a> {
    fn write_tokens(&self) -> TokenStream {
        let children_names = self.children.keys().map(|s| create_ident(s));
        let children = self.children.values().map(|module| module.write_tokens());
        let types = self.types.iter().cloned().map(TypeData::write_tokens);
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