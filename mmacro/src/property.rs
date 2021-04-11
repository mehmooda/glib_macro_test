use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn handle_property(tit: &syn::TraitItemType) -> Property {
    #[derive(Debug, Default)]
    struct PropertyAttributes {
        magic: bool,
        nick: Option<String>,
        blurb: Option<String>,
    }

    let mut attributes = PropertyAttributes::default();

    for a in &tit.attrs {
        if let Some(x) = a.path.leading_colon {
            proc_macro_error::abort!(x, "gobject_signal_properties unexpected")
        }
        if a.path.segments.len() != 1 {
            proc_macro_error::abort!(a.path, "gobject_signal_properties unexpected")
        }

        let attribute_name = &a.path.segments[0].ident;
        match format!("{}", attribute_name).as_str() {
            "property" => {
                if !a.tokens.is_empty() {
                    proc_macro_error::abort!(a.tokens, "gobject_signal_properties unexpected")
                }
                attributes.magic = true;
            }
            // #[class_handler(run_first, run_last, run_cleanup)]
            "nick" => {
                proc_macro_error::abort!(a, "gobject_signal_properties todo");
                if a.tokens.is_empty() {
                    proc_macro_error::abort!(a, "gobject_signal_properties expected nickname")
                }
                attributes.nick = Some(format!("{}", a.tokens));
            }
            "blurb" => {
                proc_macro_error::abort!(a, "gobject_signal_properties todo");

                if a.tokens.is_empty() {
                    proc_macro_error::abort!(a, "gobject_signal_properties expected blurb")
                }
                attributes.blurb = Some(format!("{}", a.tokens));
            }

            _ => proc_macro_error::abort!(a.path, "gobject_signal_properties unexpected"),
        }
    }

    let name = crate::ParamSpecName::new(&tit.ident).unwrap_or_else(|()| {
        proc_macro_error::abort!(tit.ident, "gobject_signal_properties: invalid name")
    });

    if let Some(_) = tit.generics.lt_token {
        proc_macro_error::abort!(tit.generics.params, "gobject_signal_properties unexpected")
    }

    if let Some(w) = &tit.generics.where_clause {
        proc_macro_error::abort!(w, "gobject_signal_properties unexpected")
    }

    if !tit.bounds.is_empty() {
        proc_macro_error::abort!(tit.bounds, "gobject_signal_properties unexpected")
    }

    if let None = tit.default {
        proc_macro_error::abort!(tit, "gobject_signal_properties property type needed")
    }

    Property {
        name,
        span: syn::spanned::Spanned::span(&tit),
        object: tit.default.as_ref().unwrap().1.clone(),
    }
}

pub(crate) struct Property {
    pub name: super::ParamSpecName,
    pub object: syn::Type,
    pub span: proc_macro2::Span,
}

pub(crate) fn verifications(properties: &[Property]) -> TokenStream {
    let property_type = properties.iter().map(|x| &x.object);
    quote! {
        #(
            // Verify
            verify_is_glib_StaticType::<#property_type>();
        )*
    }
}

pub(crate) fn definitions(properties: &[Property]) -> TokenStream {
    let property_type = properties.iter().map(|x| &x.object);
    let property_func_name = properties
        .iter()
        .map(|x| quote::format_ident!("get_{}", &x.name.snake_case()));
    quote! {
        #(
            fn #property_func_name (&self) -> #property_type;
        )*
    }
}

pub(crate) fn implementations(properties: &[Property]) -> TokenStream {
    let glib = super::get_glib();

    let property_type = properties.iter().map(|x| &x.object);
    let property_func_name = properties
        .iter()
        .map(|x| syn::Ident::new(&format!("get_{}", &x.name.snake_case()), x.span));
    let property_property_name = properties.iter().map(|x| x.name.kebab_case());
    quote! {
        #(
            fn #property_func_name (&self) -> #property_type {
               #glib ::ObjectExt::get_property(self, #property_property_name ).unwrap().get().unwrap().unwrap()
            }
        )*
    }
}

// TODO: Fix this so it works and uses UsableAsParam
pub(crate) fn builder(properties: &[Property]) -> TokenStream {
    let glib = super::get_glib();
    // TODO: Parse property type and call correct paramspec builder for type
    let property_property_name = properties.iter().map(|x| x.name.kebab_case());
    quote! {
        fn properties() -> &'static [#glib ::ParamSpec] {
            static PROPERTIES: #glib ::once_cell::sync::OnceCell<Vec<#glib ::ParamSpec>> = #glib ::once_cell::sync::OnceCell::new();

            PROPERTIES.get_or_init( || vec![
            #(
                #glib ::ParamSpec::string(#property_property_name,#property_property_name,#property_property_name, None, #glib ::ParamFlags::READWRITE)

            ),*])
        }
    }
}
