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
