use crate::data::*;
use crate::helpers::create_ident;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl TypeRef {
    fn write_tokens(&self) -> TokenStream {
        let name = create_ident(&self.name);
        quote! {
            #name
        }
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
        quote! {
            #[repr(C)]
            struct #name {
                #super_field,
                #( pub #fields ),*
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
