use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Lit, parse_macro_input};

/// Derive macro that reads `#[memchrn(set = "...")]` on a struct and implements
/// `MemchrNMatcher` for it with tables computed at compile time.
///
/// # Example
/// ```rust,ignore
/// #[derive(MemchrNMatcher)]
/// #[memchrn(set = "[]{}<>()")]
/// pub struct BracketMatcher;
/// ```
#[proc_macro_derive(MemchrNMatcher, attributes(memchrn))]
pub fn derive_memchr_n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_memchr_n(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_memchr_n(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Parse #[memchrn(set = "...")] attribute
    let set_str = extract_set_attr(&input.attrs)?;
    let needles: Vec<u8> = set_str.bytes().collect();

    if needles.is_empty() {
        return Err(syn::Error::new_spanned(&input.ident, "shufti set must have >=1 bytes"));
    }

    if needles.len() == 1 {
        let s1 = needles[0];
        return Ok(quote! {
            impl #impl_generics ::memchr_n::MemchrNMatch for #name #ty_generics #where_clause {
                #[inline(always)]
                fn memchr_n() -> impl ::memchr_n::MemchrN {
                    ::memchr_n::MemchrNImpl::One(#s1)
                }
            }
        });
    } else if needles.len() == 2 {
        let s1 = needles[0];
        let s2 = needles[1];
        return Ok(quote! {
            impl #impl_generics ::memchr_n::MemchrNMatch for #name #ty_generics #where_clause {
                #[inline(always)]
                fn memchr_n() -> impl ::memchr_n::MemchrN {
                    ::memchr_n::MemchrNImpl::Two(#s1, #s2)
                }
            }
        });
    } else if needles.len() == 3 {
        let s1 = needles[0];
        let s2 = needles[1];
        let s3 = needles[2];
        return Ok(quote! {
            impl #impl_generics ::memchr_n::MemchrNMatch for #name #ty_generics #where_clause {
                #[inline(always)]
                fn memchr_n() -> impl ::memchr_n::MemchrN {
                    ::memchr_n::MemchrNImpl::Three(#s1, #s2, #s3)
                }
            }
        });
    }

    let memchrn_table::ShuftiTables { low_tab, high_tab, bit_mask } =
        memchrn_table::build_tables(&needles)
            .map_err(|error| syn::Error::new_spanned(&input.ident, error))?;

    let low_tab_tokens = low_tab.iter().map(|b| quote! { #b});
    let high_tab_tokens = high_tab.iter().map(|b| quote! { #b});

    Ok(quote! {
        impl #impl_generics ::memchr_n::MemchrNMatch for #name #ty_generics #where_clause {
            #[inline(always)]
            fn memchr_n() -> impl ::memchr_n::MemchrN {
                ::memchr_n::MemchrNImpl::MemchrN( unsafe {
                    ::memchr_n::MemchrNTable::new_raw([#(#low_tab_tokens),*],
                    [#(#high_tab_tokens),*],
                    #bit_mask,)
                })
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Attribute parsing
// ---------------------------------------------------------------------------

fn extract_set_attr(attrs: &[syn::Attribute]) -> syn::Result<String> {
    for attr in attrs {
        if !attr.path().is_ident("memchrn") {
            continue;
        }

        let mut found: Option<String> = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("set") {
                let value = meta.value()?; // consumes `=`
                let lit: Lit = value.parse()?;
                if let Lit::Str(ls) = lit {
                    found = Some(ls.value());
                    Ok(())
                } else {
                    Err(meta.error("expected string literal for `set`"))
                }
            } else {
                Err(meta.error("unknown shufti attribute key"))
            }
        })?;

        if let Some(s) = found {
            return Ok(s);
        }
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "missing #[memchrn(set = \"...\")] attribute",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn derive_set(set: &str) -> syn::Result<TokenStream2> {
        let set = syn::LitStr::new(set, proc_macro2::Span::call_site());
        let input = syn::parse_quote! {
            #[memchrn(set = #set)]
            struct Matcher;
        };
        impl_memchr_n(&input)
    }

    #[test]
    fn accepts_large_ascii_sets() {
        let ascii: String = (0..=127u8).map(char::from).collect();
        assert!(derive_set(&ascii).is_ok());
        assert!(derive_set("0123456789").is_ok());
        assert!(
            derive_set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789").is_ok()
        );
    }

    #[test]
    fn rejects_sets_with_more_than_eight_patterns() {
        let error = derive_set("\x00\x11\x22\x33\x44\x55\x66\x77é").unwrap_err();
        assert_eq!(error.to_string(), "too many shufti buckets (maximum 8)");
    }

    #[test]
    fn rejects_empty_and_duplicate_table_inputs() {
        assert_eq!(derive_set("").unwrap_err().to_string(), "shufti set must have >=1 bytes");
        assert_eq!(derive_set("abcda").unwrap_err().to_string(), "needles must be unique");
        // Preserve the existing memchr behavior for up to three needles.
        assert!(derive_set("aa").is_ok());
    }
}
