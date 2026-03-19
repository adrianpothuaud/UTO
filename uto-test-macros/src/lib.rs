//! Procedural macros for UTO test authoring.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{spanned::Spanned, Expr, Lit, LitInt, LitStr, Meta, Token};

fn parse_string_array(expr: &Expr) -> Result<Vec<String>, syn::Error> {
    match expr {
        Expr::Array(arr) => {
            let mut out = Vec::new();
            for element in &arr.elems {
                match element {
                    Expr::Lit(expr_lit) => match &expr_lit.lit {
                        Lit::Str(value) => out.push(value.value()),
                        _ => {
                            return Err(syn::Error::new(
                                expr_lit.span(),
                                "tags entries must be string literals",
                            ));
                        }
                    },
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "tags entries must be string literals",
                        ));
                    }
                }
            }
            Ok(out)
        }
        _ => Err(syn::Error::new(
            expr.span(),
            "tags must be an array of string literals, e.g. tags = [\"smoke\"]",
        )),
    }
}

fn parse_timeout(expr: &Expr) -> Result<u64, syn::Error> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(value) => parse_timeout_int(value),
            _ => Err(syn::Error::new(
                expr_lit.span(),
                "timeout and timeout_ms must be integer literals",
            )),
        },
        _ => Err(syn::Error::new(
            expr.span(),
            "timeout and timeout_ms must be integer literals",
        )),
    }
}

fn parse_timeout_int(value: &LitInt) -> Result<u64, syn::Error> {
    value
        .base10_parse::<u64>()
        .map_err(|_| syn::Error::new(value.span(), "timeout value is out of range for u64"))
}

fn parse_target(expr: &Expr) -> Result<String, syn::Error> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => Ok(value.value()),
            _ => Err(syn::Error::new(
                expr_lit.span(),
                "target must be a string literal",
            )),
        },
        _ => Err(syn::Error::new(
            expr.span(),
            "target must be a string literal",
        )),
    }
}

fn parse_tags_list(meta: &Meta) -> Result<Vec<String>, syn::Error> {
    let Meta::List(list) = meta else {
        return Err(syn::Error::new(meta.span(), "expected tags(...)"));
    };

    let parser = Punctuated::<LitStr, Token![,]>::parse_terminated;
    let values = parser.parse2(list.tokens.clone())?;
    Ok(values.into_iter().map(|v| v.value()).collect())
}

fn parse_uto_test_args(attr: TokenStream) -> Result<(), syn::Error> {
    if attr.is_empty() {
        return Ok(());
    }

    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = parser.parse(attr)?;

    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("target") => {
                let _ = parse_target(&nv.value)?;
            }
            Meta::NameValue(nv) if nv.path.is_ident("tags") => {
                let _ = parse_string_array(&nv.value)?;
            }
            Meta::NameValue(nv)
                if nv.path.is_ident("timeout") || nv.path.is_ident("timeout_ms") =>
            {
                let _ = parse_timeout(&nv.value)?;
            }
            Meta::List(list) if list.path.is_ident("tags") => {
                let _ = parse_tags_list(&Meta::List(list))?;
            }
            Meta::NameValue(nv) => {
                return Err(syn::Error::new(
                    nv.path.span(),
                    "unsupported uto_test argument; supported: target, tags, timeout, timeout_ms",
                ));
            }
            Meta::Path(path) => {
                return Err(syn::Error::new(
                    path.span(),
                    "unsupported uto_test flag; use key-value arguments",
                ));
            }
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.path.span(),
                    "unsupported uto_test argument list; supported: tags(...) only",
                ));
            }
        }
    }

    Ok(())
}

/// Marks a test function as a UTO framework test.
///
/// Current MVP behavior keeps the function unchanged while allowing UTO to
/// standardize test annotation patterns across generated projects and examples.
#[proc_macro_attribute]
pub fn uto_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Err(error) = parse_uto_test_args(attr) {
        return TokenStream::from(error.to_compile_error());
    }

    let item_ts: proc_macro2::TokenStream = item.into();
    TokenStream::from(quote! { #item_ts })
}
