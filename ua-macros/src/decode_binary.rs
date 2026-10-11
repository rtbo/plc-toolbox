use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Fields, Path, Token};

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
                "DecodeBinary cannot be derived for unions",
            ));
        }
    }
}

pub fn expand_struct(
    name: syn::Ident,
    mut generics: syn::Generics,
    fields: syn::Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_accesses: Vec<_> = match &fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("named field has an identifier");
                syn::Member::Named(ident.clone())
            })
            .collect(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, _)| syn::Member::Unnamed(index.into()))
            .collect(),
        syn::Fields::Unit => Vec::new(),
    };

    for field in &fields {
        let field_type = &field.ty;
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(#field_type: ::ua_core::msg::binary::DecodeBinary));
    }

    let decode_fields = field_accesses.iter().map(|member| {
        quote! {
            #member: ::ua_core::msg::binary::DecodeBinary::decode(buf)?
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::ua_core::msg::binary::DecodeBinary for #name #ty_generics #where_clause {
            fn decode<B: ::bytes::Buf>(buf: &mut B) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
                Ok(Self {
                    #(#decode_fields),*
                })
            }
        }
    };

    Ok(expanded)
}

fn expand_enum(
    name: syn::Ident,
    generics: syn::Generics,
    attrs: &[syn::Attribute],
    data: syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let repr_type = enum_repr_type(&name, attrs)?;

    let variant_names = data
        .variants
        .iter()
        .map(|variant| {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    &variant.fields,
                    "DecodeBinary only supports fieldless enums",
                ));
            }
            Ok(&variant.ident)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let try_from_arms = variant_names.iter().map(|variant| {
        quote! {
            value if value == Self::#variant as #repr_type => Ok(Self::#variant)
        }
    });
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::convert::TryFrom<#repr_type> for #name #ty_generics #where_clause {
            type Error = ();

            fn try_from(value: #repr_type) -> Result<Self, Self::Error> {
                match value {
                    #(#try_from_arms,)*
                    _ => Err(()),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::ua_core::msg::binary::DecodeBinary for #name #ty_generics #where_clause {
            fn decode<B: ::ua_core::msg::binary::Buf>(
                buf: &mut B,
            ) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
                let value = <#repr_type as ::ua_core::msg::binary::DecodeBinary>::decode(buf)?;
                Self::try_from(value)
                    .map_err(|_| ::ua_core::msg::binary::DecodingError::InvalidData)
            }
        }
    })
}

fn enum_repr_type(name: &syn::Ident, attrs: &[Attribute]) -> syn::Result<Path> {
    let mut repr_type: Option<Path> = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("repr")) {
        let reprs = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
        for repr in reprs {
            if repr.is_ident("u8") || repr.is_ident("u16") || repr.is_ident("u32") {
                if repr_type.is_some() {
                    return Err(syn::Error::new_spanned(
                        repr,
                        "DecodeBinary enums must have a single integer repr",
                    ));
                }
                repr_type = Some(repr);
            }
        }
    }

    repr_type.ok_or_else(|| {
        syn::Error::new_spanned(
            name,
            "DecodeBinary enums require #[repr(u8)], #[repr(u16)], or #[repr(u32)]",
        )
    })
}
