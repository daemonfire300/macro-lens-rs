//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//
// Copyright (c) 2025 Julius Foitzik on derivative work
// All rights reserved.
//

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Expr, ExprPath, Member, parse_macro_input};

#[derive(Clone, Debug)]
enum LensStep {
    Field(syn::Ident),
    Index(Box<Expr>),
}

#[derive(Debug)]
struct ParsedLens {
    root: syn::Ident,
    steps: Vec<LensStep>,
}

#[proc_macro]
pub fn lens(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    match parse_lens_expression(&expr).and_then(expand_lens_expression) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => error.to_compile_error().into(),
    }
}

fn parse_lens_expression(expr: &Expr) -> Result<ParsedLens, syn::Error> {
    let mut steps = Vec::new();
    let root = collect_steps(expr, &mut steps)?;
    if steps.is_empty() {
        return Err(syn::Error::new(
            expr.span(),
            "lens!() expression must include at least one field access",
        ));
    }

    Ok(ParsedLens { root, steps })
}

fn collect_steps(expr: &Expr, steps: &mut Vec<LensStep>) -> Result<syn::Ident, syn::Error> {
    match expr {
        Expr::Field(field_access) => {
            let root = collect_steps(&field_access.base, steps)?;
            let field_name = match &field_access.member {
                Member::Named(field_ident) => field_ident.clone(),
                Member::Unnamed(_) => {
                    return Err(syn::Error::new(
                        field_access.span(),
                        "lens!() only works with named fields, not tuple indexing",
                    ));
                }
            };
            steps.push(LensStep::Field(field_name));
            Ok(root)
        }
        Expr::Index(index_expr) => {
            let root = collect_steps(&index_expr.expr, steps)?;
            steps.push(LensStep::Index(index_expr.index.clone()));
            Ok(root)
        }
        Expr::Path(path) => parse_root(path),
        _ => Err(syn::Error::new(
            expr.span(),
            "lens!() expression must look like `Root.field` or `Root.field[index].child`",
        )),
    }
}

fn parse_root(path: &ExprPath) -> Result<syn::Ident, syn::Error> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new(
            path.span(),
            "lens!() expression must start with an unqualified struct name",
        ));
    }

    Ok(path.path.segments[0].ident.clone())
}

fn expand_lens_expression(parsed: ParsedLens) -> Result<TokenStream2, syn::Error> {
    let mut lens_exprs = Vec::with_capacity(parsed.steps.len());
    let root_lenses = format_ident!("_{}Lenses", parsed.root);
    let mut current_lenses_expr = quote!(#root_lenses);
    let mut last_field_context: Option<FieldContext> = None;

    for step in parsed.steps {
        match step {
            LensStep::Field(field_name) => {
                let field_lenses_expr = current_lenses_expr.clone();
                let field_expr = quote!(#field_lenses_expr.#field_name);
                lens_exprs.push(field_expr.clone());
                let item_marker_name = vec_item_marker_name(&field_name);
                let item_lenses_name = vec_item_lenses_field_name(&field_name);
                let nested_lenses_name = nested_lenses_field_name(&field_name);

                last_field_context = Some(FieldContext {
                    item_marker_expr: quote!(#field_lenses_expr.#item_marker_name),
                    item_lenses_expr: quote!(#field_lenses_expr.#item_lenses_name),
                });
                current_lenses_expr = quote!(#field_lenses_expr.#nested_lenses_name);
            }
            LensStep::Index(index_expr) => {
                let Some(field_context) = &last_field_context else {
                    return Err(syn::Error::new(
                        index_expr.span(),
                        "lens!() indexing is only supported immediately after a named field",
                    ));
                };
                let item_marker_expr = field_context.item_marker_expr.clone();
                let vec_expr = quote!(lens::vec_lens_from_marker(#item_marker_expr, #index_expr));
                lens_exprs.push(vec_expr);
                current_lenses_expr = field_context.item_lenses_expr.clone();
                last_field_context = None;
            }
        }
    }

    Ok(quote! {
        lens::compose_lens!(#(#lens_exprs),*)
    })
}

struct FieldContext {
    item_marker_expr: TokenStream2,
    item_lenses_expr: TokenStream2,
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parses_field_and_index_steps() {
        let expr: Expr = syn::parse2(quote!(Root.items[1].value)).expect("valid expression");
        let parsed = parse_lens_expression(&expr).expect("parsed lens expression");
        assert_eq!(parsed.root.to_string(), "Root");
        assert_eq!(parsed.steps.len(), 3);
        assert!(matches!(parsed.steps[0], LensStep::Field(_)));
        assert!(matches!(parsed.steps[1], LensStep::Index(_)));
        assert!(matches!(parsed.steps[2], LensStep::Field(_)));
    }

    #[test]
    fn rejects_qualified_roots() {
        let expr: Expr = syn::parse2(quote!(crate::Root.value)).expect("valid expression");
        let error = parse_lens_expression(&expr).expect_err("qualified root should fail");
        assert!(
            error
                .to_string()
                .contains("must start with an unqualified struct name")
        );
    }

    #[test]
    fn rejects_tuple_fields() {
        let expr: Expr = syn::parse2(quote!(Root.0)).expect("valid expression");
        let error = parse_lens_expression(&expr).expect_err("tuple field should fail");
        assert!(error.to_string().contains("named fields"));
    }
}
