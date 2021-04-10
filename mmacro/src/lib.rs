use proc_macro::TokenStream;
use quote::format_ident;

mod property;
mod signal;
mod util;

use util::*;

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn gobject_signal_properties(attr: TokenStream, _item: TokenStream) -> TokenStream {
    // Should we allow renaming of {}Ext and {}Builder items?
    handle_gsp_attributes(attr);

    let parsed: Result<syn::ItemTrait, _> = syn::parse(_item);
    if let Err(y) = parsed {
        return y.into_compile_error().into();
    }

    let item_trait = parsed.unwrap();
    ensure_gsp_is_simple_trait(&item_trait);

    let (signals, properties) = gsp_parse_signals_and_properties(&item_trait);

    let glib = get_glib();

    let pverify = property::verifications(&properties);
    let pdef = property::definitions(&properties);
    let pimpl = property::implementations(&properties);
    let pbuilder = property::builder(&properties);

    let object = item_trait.ident;
    let objectext = format_ident!("{}Ext", object);
    let objectbuilder = format_ident!("{}ObjectSubclassBuilder", object);
    let impl_mod = format_ident!("__impl_gobject_properties_{}", object);

    quote::quote!(
            //TODO: GivePropertyBuilderName
            struct #objectbuilder;
            impl #objectbuilder {
                #pbuilder
    //          #sbuilder
            }

            // Will ensure object is a defined type or error;
            impl #object {}

            mod #impl_mod {
                #![allow(non_snake_case)]
                fn verify_is_glib_object<T: #glib::IsA<#glib::Object>>(){}
                fn verify_is_glib_FromValueOptional<'a, T: #glib::value::FromValueOptional<'a>>(){}
                fn verify_is_glib_ToValueOptional<T: #glib::value::ToValue>(){}

                fn test() {
                    verify_is_glib_object::<super::#object>();
                    #pverify
    //                #sverify
                }
            }

            trait #objectext {
                #pdef
    //            #sdef
            }

            impl<T: #glib::IsA<#object>> #objectext for T {
                #pimpl
    //            #simpl
            }
        )
    .into()
}

fn handle_gsp_attributes(attr: TokenStream) {
    if !attr.is_empty() {
        proc_macro_error::abort_call_site!(
            "gobject_signal_properties does not support extra arguments"
        );
    }
}

fn ensure_gsp_is_simple_trait(item_trait: &syn::ItemTrait) {
    if item_trait.attrs.len() != 0 {
        proc_macro_error::abort!(
            item_trait.attrs[0],
            "gobject_signal_properties unexpected attribute"
        )
    }
    if let Some(u) = item_trait.unsafety {
        proc_macro_error::abort!(u, "gobject_signal_properties unexpected")
    }
    if let Some(a) = item_trait.auto_token {
        proc_macro_error::abort!(a, "gobject_signal_properties unexpected")
    }
    if let Some(_) = item_trait.generics.lt_token {
        proc_macro_error::abort!(
            item_trait.generics.params,
            "gobject_signal_properties unexpected"
        )
    }
    if let Some(w) = &item_trait.generics.where_clause {
        proc_macro_error::abort!(w, "gobject_signal_properties unexpected")
    }
    if let Some(_) = item_trait.colon_token {
        proc_macro_error::abort!(
            item_trait.supertraits,
            "gobject_signal_properties unexpected"
        )
    }
}

fn gsp_parse_signals_and_properties(
    item_trait: &syn::ItemTrait,
) -> (Vec<signal::Signal>, Vec<property::Property>) {
    let mut signals = vec![];
    let mut properties = vec![];

    for x in &item_trait.items {
        use syn::TraitItem;
        //        println!("{:?}", x);
        match x {
            TraitItem::Method(tim) => signals.push(signal::handle_signal(tim)),
            TraitItem::Type(tit) => properties.push(property::handle_property(tit)),
            TraitItem::Macro(_) | TraitItem::Const(_) | TraitItem::Verbatim(_) => {
                proc_macro_error::abort!(x, "gobject_signal_properties unexpected")
            }
            #[cfg(not(test))]
            _ => proc_macro_error::abort!(x, "gobject_signal_properties unexpected"),
            #[cfg(test)]
            TraitItem::__TestExhaustive(_) => unimplemented!(),
        };
    }

    (signals, properties)
}
