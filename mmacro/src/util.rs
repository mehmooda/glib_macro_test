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

pub fn get_glib() -> proc_macro2::TokenStream {
    use proc_macro_crate::*;

    let found_crate = crate_name("glib");
    if let Ok(s) = found_crate {
        let s = syn::Ident::new(&s, proc_macro2::Span::call_site());
        return quote::quote!( #s );
    }

    let found_crate = crate_name("gtk4");
    if let Ok(s) = found_crate {
        let s = syn::Ident::new(&s, proc_macro2::Span::call_site());
        return quote::quote!( ::#s::glib );
    }

    panic!("Unable to find glib")
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
