use crate::data::*;
use crate::helpers::create_ident;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

fn get_qualified_name(namespace: &str, name: &str) -> TokenStream {
    let namespace_tokens = namespace.split_terminator('.').map(create_ident);
    let name_ident = create_ident(name);
    quote! {
        #( #namespace_tokens :: )* #name_ident
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
            impl Deref for #name {
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
                #super_ident: #super_type
            }
        });
        let deref = self.write_deref(&name);
        let methods = self.methods.iter().map(Method::write_tokens);
        quote! {
            #[repr(C)]
            struct #name {
                #super_field,
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
            TypeEnum::Class => self.write_class(),
            _ => quote! {},
        }
    }
}

impl DllData {
    pub fn write_tokens(&self) -> TokenStream {
        let types = self.types.iter().map(TypeData::write_tokens);

        quote! {
            #( #types )*
        }
    }
}
