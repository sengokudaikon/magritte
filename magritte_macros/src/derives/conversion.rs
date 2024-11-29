use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, Lit};

pub fn lit_to_tokens(lit: &Lit) -> TokenStream {
    match lit {
        Lit::Str(s) => quote!(#s),
        _ => {
            let s = lit.clone();
            quote!(stringify!(#s))
        }
    }
}

pub fn expr_to_tokens(expr: &Expr) -> TokenStream {
    quote!(#expr)
}

pub fn expr_array_to_vec(array: &ExprArray) -> TokenStream {
    let elements = array.elems.iter();
    quote! {
        vec![#(#elements),*]
    }
}
pub fn parse_number<T: std::str::FromStr>(lit: &Lit) -> Option<T> {
    match lit {
        Lit::Int(i) => i.base10_digits().parse().ok(),
        Lit::Float(f) => f.base10_digits().parse().ok(),
        _ => None,
    }
}
