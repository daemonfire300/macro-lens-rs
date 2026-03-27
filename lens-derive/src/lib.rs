//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
// Copyright (c) 2025 Julius Foitzik on derivative work.
// All rights reserved.
//

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, PathArguments, Type, Visibility,
    parse_macro_input,
};

/// Handles the `#[derive(Lenses)]` applied to a struct by generating a `Lens` implementation for
/// each field in the struct.
#[proc_macro_derive(Lenses)]
pub fn lenses_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_lenses(&input) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_lenses(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "`#[derive(Lenses)]` may only be applied to structs",
            ));
        }
    };

    let fields = match &data_struct.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "`#[derive(Lenses)]` may only be applied to structs with named fields",
            ));
        }
    };

    let struct_name = &input.ident;
    let lens_visibility = &input.vis;

    let lens_items = fields
        .iter()
        .enumerate()
        .map(|(index, field)| expand_field_lens(struct_name, lens_visibility, index as u64, field))
        .collect::<Result<Vec<_>, _>>()?;

    let lenses_struct_name = format_ident!("{struct_name}Lenses");
    let lenses_struct_fields = fields
        .iter()
        .map(|field| expand_lenses_struct_fields(struct_name, field))
        .collect::<Result<Vec<_>, _>>()?;

    let lenses_const_name = format_ident!("_{struct_name}Lenses");
    let lenses_const_fields = fields
        .iter()
        .map(|field| expand_lenses_const_fields(struct_name, field))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #(#lens_items)*

        #[allow(dead_code)]
        #[doc(hidden)]
        #lens_visibility struct #lenses_struct_name {
            #(#lenses_struct_fields),*
        }

        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        #lens_visibility const #lenses_const_name: #lenses_struct_name = #lenses_struct_name {
            #(#lenses_const_fields),*
        };
    })
}

fn expand_field_lens(
    struct_name: &syn::Ident,
    lens_visibility: &Visibility,
    field_index: u64,
    field: &Field,
) -> Result<TokenStream2, syn::Error> {
    let field_name = field_name(field)?;
    let field_type = &field.ty;
    let lens_name = lens_type_name(struct_name, field_name);
    let value_lens = if is_value_lens_type(field_type) {
        quote! {
            #[allow(dead_code)]
            impl lens::ValueLens for #lens_name {
                #[inline(always)]
                fn get(&self, source: &#struct_name) -> #field_type {
                    (*source).#field_name.clone()
                }
            }
        }
    } else {
        quote!()
    };

    Ok(quote! {
        #[allow(dead_code)]
        #[doc(hidden)]
        #lens_visibility struct #lens_name;

        #[allow(dead_code)]
        impl lens::Lens for #lens_name {
            type Source = #struct_name;
            type Target = #field_type;

            #[inline(always)]
            fn path(&self) -> lens::LensPath {
                lens::LensPath::new(#field_index)
            }

            #[inline(always)]
            fn mutate(&self, source: &mut #struct_name, target: #field_type) {
                source.#field_name = target
            }
        }

        #[allow(dead_code)]
        impl lens::RefLens for #lens_name {
            #[inline(always)]
            fn get_ref<'a>(&self, source: &'a #struct_name) -> &'a #field_type {
                &(*source).#field_name
            }

            #[inline(always)]
            fn get_mut_ref<'a>(&self, source: &'a mut #struct_name) -> &'a mut #field_type {
                &mut (*source).#field_name
            }
        }

        #value_lens
    })
}

fn expand_lenses_struct_fields(
    struct_name: &syn::Ident,
    field: &Field,
) -> Result<TokenStream2, syn::Error> {
    let field_name = field_name(field)?;
    let field_lens_name = lens_type_name(struct_name, field_name);
    let mut generated = vec![quote!(#field_name: #field_lens_name)];

    if let Some(item_type) = vec_item_type(&field.ty) {
        let item_marker_name = vec_item_marker_name(field_name);
        generated.push(quote!(#item_marker_name: std::marker::PhantomData<#item_type>));
        if !is_value_lens_type(item_type) {
            let item_lenses_name = vec_item_lenses_field_name(field_name);
            let item_lenses_type_name = nested_lenses_type_name(item_type)?;
            generated.push(quote!(#item_lenses_name: #item_lenses_type_name));
        }
    } else if !is_value_lens_type(&field.ty) {
        let field_parent_lenses_field_name = nested_lenses_field_name(field_name);
        let field_parent_lenses_type_name = nested_lenses_type_name(&field.ty)?;
        generated.push(quote!(
            #field_parent_lenses_field_name: #field_parent_lenses_type_name
        ));
    }

    Ok(quote!(#(#generated),*))
}

fn expand_lenses_const_fields(
    struct_name: &syn::Ident,
    field: &Field,
) -> Result<TokenStream2, syn::Error> {
    let field_name = field_name(field)?;
    let field_lens_name = lens_type_name(struct_name, field_name);
    let mut generated = vec![quote!(#field_name: #field_lens_name)];

    if let Some(item_type) = vec_item_type(&field.ty) {
        let item_marker_name = vec_item_marker_name(field_name);
        generated.push(quote!(#item_marker_name: std::marker::PhantomData));
        if !is_value_lens_type(item_type) {
            let item_lenses_name = vec_item_lenses_field_name(field_name);
            let item_lenses_type_name = nested_lenses_const_name(item_type)?;
            generated.push(quote!(#item_lenses_name: #item_lenses_type_name));
        }
    } else if !is_value_lens_type(&field.ty) {
        let field_parent_lenses_field_name = nested_lenses_field_name(field_name);
        let field_parent_lenses_type_name = nested_lenses_const_name(&field.ty)?;
        generated.push(quote!(
            #field_parent_lenses_field_name: #field_parent_lenses_type_name
        ));
    }

    Ok(quote!(#(#generated),*))
}

fn field_name(field: &Field) -> Result<&syn::Ident, syn::Error> {
    field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            field,
            "`#[derive(Lenses)]` may only be applied to structs with named fields",
        )
    })
}

fn lens_type_name(struct_name: &syn::Ident, field_name: &syn::Ident) -> syn::Ident {
    format_ident!(
        "{}{}Lens",
        struct_name,
        to_camel_case(&field_name.to_string())
    )
}

fn nested_lenses_field_name(field_name: &syn::Ident) -> syn::Ident {
    format_ident!("{field_name}_lenses")
}

fn vec_item_marker_name(field_name: &syn::Ident) -> syn::Ident {
    format_ident!("{field_name}_item")
}

fn vec_item_lenses_field_name(field_name: &syn::Ident) -> syn::Ident {
    format_ident!("{field_name}_item_lenses")
}

fn nested_lenses_type_name(ty: &Type) -> Result<syn::Ident, syn::Error> {
    let ident = terminal_type_ident(ty)?;
    Ok(format_ident!("{ident}Lenses"))
}

fn nested_lenses_const_name(ty: &Type) -> Result<syn::Ident, syn::Error> {
    let ident = terminal_type_ident(ty)?;
    Ok(format_ident!("_{ident}Lenses"))
}

fn terminal_type_ident(ty: &Type) -> Result<syn::Ident, syn::Error> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.clone())
            .ok_or_else(|| syn::Error::new_spanned(ty, "unsupported field type for `Lenses`")),
        _ => Err(syn::Error::new_spanned(
            ty,
            "unsupported field type for `Lenses`",
        )),
    }
}

fn vec_item_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }

    match arguments.args.first()? {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn is_value_lens_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    matches!(
        segment.ident.to_string().as_str(),
        "bool"
            | "char"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "String"
    )
}

fn to_camel_case(s: &str) -> String {
    s.split('_')
        .flat_map(|word| {
            word.chars().enumerate().map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect()
                }
            })
        })
        .collect::<Vec<_>>()
        .concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn to_camel_case_should_work() {
        assert_eq!(to_camel_case("this_is_snake_case"), "ThisIsSnakeCase");
    }

    #[test]
    fn vec_item_type_should_detect_vec_fields() {
        let ty: Type = syn::parse2(quote!(Vec<MyStruct>)).expect("valid type");
        let item_type = vec_item_type(&ty).expect("vec item type");
        assert_eq!(quote!(#item_type).to_string(), "MyStruct");
    }

    #[test]
    fn scalar_types_should_get_value_lenses() {
        let ty: Type = syn::parse2(quote!(String)).expect("valid type");
        assert!(is_value_lens_type(&ty));
    }

    #[test]
    fn nested_type_name_should_use_the_actual_field_type() {
        let ty: Type = syn::parse2(quote!(crate::models::Address)).expect("valid type");
        assert_eq!(
            nested_lenses_type_name(&ty)
                .expect("nested lenses type")
                .to_string(),
            "AddressLenses"
        );
    }
}
