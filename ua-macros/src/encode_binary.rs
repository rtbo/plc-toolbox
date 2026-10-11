use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Fields, Generics, Path, Token, parse_quote};

pub fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        ident: name,
        generics,
        data,
        attrs,
        ..
    } = input;

    match data {
        Data::Struct(data) => expand_struct(name, generics, data.fields),
        Data::Enum(data) => expand_enum(name, generics, &attrs, data),
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "EncodeBinary cannot be derived for unions",
            ));
        }
    }
}

fn expand_struct(
    name: syn::Ident,
    mut generics: Generics,
    fields: Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_accesses: Vec<_> = match &fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("named field has an identifier");
                syn::Member::Named(ident.clone())
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, _)| syn::Member::Unnamed(index.into()))
            .collect(),
        Fields::Unit => Vec::new(),
    };

    for field in &fields {
        let field_type = &field.ty;
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#field_type: ::ua_core::msg::binary::EncodeBinary));
    }

    let encoded_len_fields = field_accesses.iter().map(|member| {
        quote! {
            ::ua_core::msg::binary::EncodeBinary::encoded_len(&self.#member)
        }
    });
    let encode_fields = field_accesses.iter().map(|member| {
        quote! {
            ::ua_core::msg::binary::EncodeBinary::encode(&self.#member, buf);
        }
    });
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::ua_core::msg::binary::EncodeBinary for #name #ty_generics #where_clause {
            fn encoded_len(&self) -> u32 {
                0 #( + #encoded_len_fields )*
            }

            fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
                #(#encode_fields)*
            }
        }
    })
}

fn expand_enum(
    name: syn::Ident,
    generics: Generics,
    attrs: &[Attribute],
    data: syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut repr_type: Option<Path> = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("repr")) {
        let reprs = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
        for repr in reprs {
            if repr.is_ident("u8") || repr.is_ident("u16") || repr.is_ident("u32") {
                if repr_type.is_some() {
                    return Err(syn::Error::new_spanned(
                        repr,
                        "EncodeBinary enums must have a single integer repr",
                    ));
                }
                repr_type = Some(repr);
            }
        }
    }

    let repr_type = repr_type.ok_or_else(|| {
        syn::Error::new_spanned(
            &name,
            "EncodeBinary enums require #[repr(u8)], #[repr(u16)], or #[repr(u32)]",
        )
    })?;

    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.fields,
                "EncodeBinary only supports fieldless enums",
            ));
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::ua_core::msg::binary::EncodeBinary for #name #ty_generics #where_clause {
            fn encoded_len(&self) -> u32 {
                let value: #repr_type = *self as #repr_type;
                ::ua_core::msg::binary::EncodeBinary::encoded_len(&value)
            }

            fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
                let value: #repr_type = *self as #repr_type;
                ::ua_core::msg::binary::EncodeBinary::encode(&value, buf);
            }
        }
    })
}
