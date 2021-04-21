pub trait TurnOptionIntoInner {
    fn is_option(&self) -> bool;
    fn get_inner_type(&self) -> &Self;
    fn inner_if_option(&self) -> &Self {
        if self.is_option() {
            self.get_inner_type()
        } else {
            self
        }
    }
}

impl TurnOptionIntoInner for syn::Type {
    fn is_option(&self) -> bool {
        if let syn::Type::Path(tp) = self {
            if tp.path.segments[0].ident == "Option" {
                return true;
            }
        }
        false
    }

    fn get_inner_type(&self) -> &Self {
        if let syn::Type::Path(tp) = self {
            if tp.path.segments[0].ident != "Option" {
                proc_macro_error::abort!(self, "Not Option")
            }
            if let syn::PathArguments::AngleBracketed(abga) = &tp.path.segments[0].arguments {
                if let syn::GenericArgument::Type(inner) = &abga.args[0] {
                    return &inner;
                }
                proc_macro_error::abort!(self, "Generic Inner Type Error")
            }
            proc_macro_error::abort!(self, "PathArguments::AngleBracketed Error")
        }
        proc_macro_error::abort!(self, "Not a Path")
    }
}

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use syn::Ident;

const KNOWN_GLIB_EXPORTS: [&'static str; 14] = [
    // Current Re-exports from gtk-rs
    "gtk",
    "gio",
    "gdk",
    "gdk-pixbuf",
    "gdkx11",
    "graphene",
    "pango",
    "pangocairo",
    // Current Re-exports from gtk4-rs
    "gtk4",
    "gsk4",
    "gdk4",
    "gdk4-wayland",
    "gdk4-x11",
    // Special Request
    "gstreamer",
];

pub fn crate_ident_new() -> TokenStream {
    use proc_macro_crate::FoundCrate;

    let crate_path = match crate_name("glib") {
        Ok(FoundCrate::Name(name)) => Some(quote!(::#name)),
        Ok(FoundCrate::Itself) => Some(quote!(::glib)),
        Err(_) => None,
    };

    let crate_path = crate_path.or_else(|| {
        KNOWN_GLIB_EXPORTS.iter().find_map(|c| {
            if let Ok(f) = crate_name(c) {
                let crate_name = match f {
                    FoundCrate::Name(name) => name,
                    FoundCrate::Itself => c.to_string(),
                };
                let crate_root = Ident::new(&crate_name, Span::call_site());
                Some(quote::quote! {
                    ::#crate_root::glib
                })
            } else {
                None
            }
        })
    });

    crate_path.unwrap_or_else(|| {
        proc_macro_error::emit_call_site_warning!(
            "Can't find glib crate. Please ensure you have a glib in scope"
        );
        let glib = Ident::new("glib", Span::call_site());
        quote!(#glib)
    })
}

pub fn get_glib() -> proc_macro2::TokenStream {
    crate_ident_new()
}

pub struct ParamSpecName(String);

#[allow(dead_code, unreachable_code)]
impl ParamSpecName {
    pub fn new(name: &syn::Ident) -> Result<Self, ()> {
        let name = format!("{}", name);

        if !name.as_bytes().iter().enumerate().all(|(i, c)| {
            i != 0 && (*c >= b'0' && *c <= b'9' || *c == b'_')
                || (*c >= b'A' && *c <= b'Z')
                || (*c >= b'a' && *c <= b'z')
        }) {
            return Err(());
        }

        Ok(ParamSpecName(name))
    }
    pub fn snake_case(&self) -> &str {
        &self.0
    }
    pub fn kebab_case(&self) -> String {
        self.0.replace("_", "-")
    }
}
