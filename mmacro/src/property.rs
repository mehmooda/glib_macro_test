use proc_macro2::TokenStream;
use quote::quote;

use crate::util::TurnOptionIntoInner;

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
                // if a.tokens.is_empty() {
                // proc_macro_error::abort!(a, "gobject_signal_properties expected nickname")
                // }
                // attributes.nick = Some(format!("{}", a.tokens));
            }
            "blurb" => {
                proc_macro_error::abort!(a, "gobject_signal_properties todo");

                // if a.tokens.is_empty() {
                // proc_macro_error::abort!(a, "gobject_signal_properties expected blurb")
                // }
                // attributes.blurb = Some(format!("{}", a.tokens));
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
        prop_type: tit.default.as_ref().unwrap().1.clone(),
    }
}

pub(crate) struct Property {
    pub name: super::ParamSpecName,
    pub prop_type: syn::Type,
    pub span: proc_macro2::Span,
}

impl Property {
    fn param_spec_constructor(&self) -> TokenStream {
        let type_path_ident = if let syn::Type::Path(tp) = &self.prop_type {
            match &tp.path.segments[0].ident {
                x => x,
            }
        } else {
            proc_macro_error::abort!(self.span, "Expected A type path")
        };

        let pname = self.name.kebab_case();

        match type_path_ident.to_string().as_str() {
            "bool" => quote! {boolean(#pname,#pname,#pname, false, ParamFlags::READWRITE)},
            "i8" => quote! {char(#pname,#pname,#pname, i8::MIN, i8::MAX, 0, ParamFlags::READWRITE)},
            "u8" => {
                quote! {uchar(#pname,#pname,#pname, u8::MIN, u8::MAX, 0, ParamFlags::READWRITE)}
            }
            "i32" => {
                quote! {int(#pname,#pname,#pname, i32::MIN, i32::MAX, 0, ParamFlags::READWRITE)}
            }
            "u32" => {
                quote! {uint(#pname,#pname,#pname, u32::MIN, u32::MAX, 0, ParamFlags::READWRITE)}
            }
            "i64" => {
                quote! {int64(#pname,#pname,#pname, i64::MIN, i64::MAX, 0, ParamFlags::READWRITE)}
            }
            "u64" => {
                quote! {uint64(#pname,#pname,#pname, u64::MIN, u64::MAX, 0, ParamFlags::READWRITE)}
            }
            "f32" => {
                quote! {floar(#pname,#pname,#pname, f32::MIN, f32::MAX, 0, ParamFlags::READWRITE)}
            }
            "f64" => {
                quote! {double(#pname,#pname,#pname, f64::MIN, f64::MAX, 0, ParamFlags::READWRITE)}
            }
            "String" => quote! {string(#pname,#pname,#pname, None, ParamFlags::READWRITE)},
            // long > c_long
            // boxed
            // enum_
            // flags
            // gtype
            // param
            // pointer
            // string
            // unichar
            // value_array
            // variant
            _ => {
                quote! {object(#pname,#pname,#pname, <#type_path_ident as StaticType>::static_type(), ParamFlags::READWRITE)}
            }
        }
    }
}

pub(crate) fn verifications(properties: &[Property]) -> TokenStream {
    let property_type = properties.iter().map(|x| x.prop_type.inner_if_option());
    quote! {
        #(
            // Verify
            verify_is_glib_StaticType::<#property_type>();
        )*
    }
}

pub(crate) fn definitions(properties: &[Property]) -> TokenStream {
    let property_type = properties.iter().map(|x| &x.prop_type);
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

    let property_type = properties.iter().map(|x| &x.prop_type);
    let property_func_name = properties
        .iter()
        .map(|x| syn::Ident::new(&format!("get_{}", &x.name.snake_case()), x.span));
    let pname = properties.iter().map(|x| x.name.kebab_case());
    quote! {
        #(
            fn #property_func_name (&self) -> #property_type {
               #glib ::ObjectExt::get_property(self, #pname ).unwrap().get().unwrap().unwrap()
            }
        )*
    }
}

// TODO: Fix this so it works and uses UsableAsParam
pub(crate) fn builder(properties: &[Property]) -> TokenStream {
    let glib = super::get_glib();
    // TODO: Parse property type and call correct paramspec builder for type
    let constructor = properties.iter().map(Property::param_spec_constructor);
    quote! {
        fn properties() -> &'static [#glib ::ParamSpec] {
            use #glib::ParamFlags;
            use #glib::StaticType;
            static PROPERTIES: #glib ::once_cell::sync::OnceCell<Vec<#glib ::ParamSpec>> = #glib ::once_cell::sync::OnceCell::new();

            PROPERTIES.get_or_init( || vec![
            #(
                #glib ::ParamSpec:: #constructor

            ),*])
        }
    }
}
