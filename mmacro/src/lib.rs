use proc_macro::TokenStream;
use quote::{format_ident, quote};

mod property;
mod signal;
mod util;

#[cfg(test)]
mod test;

use util::*;

/// # Usage
/// ```ignore
/// #[gobject_signal_properties]
/// trait NewObject {
///    #[signal]
///    fn new_signal(&self, arg: u64, second_arg: glib::Object);
///    #[property]
///    type new_property = u64;
/// }
/// ```
/// # Generated Code
/// ```ignore
/// # use glib::Object as NewObject;
/// struct NewObjectObjectSubClassBuilder {}
/// impl NewObjectObjectSubClassBuilder {
///     fn signals() -> &'static [Signal] {
///         ...
///     }
///     fn properties() -> &'static [ParamSpec] {
///         ...
///     }
/// }
/// trait NewObjectExt {
///     fn connect_new_signal<F: Fn(&Self, u64, glib::Object) + 'static>(
///         &self,
///         signal_handler: F
///     ) -> SignalHandlerId;
///
///     fn emit_new_signal(&self, arg: u64, second_arg: glib::Object);
///
///     fn get_new_property(&self) -> u64;
///
///     fn set_new_property(&self, arg: u64);
/// }
/// impl<T: IsA<NewObject>> NewObjectExt for T {
///     ...
/// }
/// ```
/// There is also some test code generated which verifies that all types in the
/// signals and properties are types supported as
/// [`StaticType`](https://docs.rs/glib/0.10.3/glib/types/trait.StaticType.html)
/// and can be converted to and from
/// [`Value`](https://docs.rs/glib/0.10.3/glib/value/struct.Value.html)
/// # Limitations
/// Property types must be a fundamental type or Object Subclass. See
/// [source](../src/mmacro/property.rs.html#99) for all currently supported
/// types

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn gobject_signal_properties(attr: TokenStream, _item: TokenStream) -> TokenStream {
    gobject_signal_properties_impl(attr.into(), _item.into()).into()
}

fn gobject_signal_properties_impl(
    attr: proc_macro2::TokenStream,
    _item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // Should we allow renaming of {}Ext and {}Builder items?
    handle_gsp_attributes(attr);

    let parsed: Result<syn::ItemTrait, _> = syn::parse2(_item);
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

    let sverify = signal::verifications(&signals);
    let sdef = signal::definitions(&signals);
    let simpl = signal::implementations(&signals);
    let sbuilder = signal::builder(&signals);

    let object = item_trait.ident;
    let objectext = format_ident!("{}Ext", object);
    let objectbuilder = format_ident!("{}ObjectSubclassBuilder", object);
    let impl_mod = format_ident!("__impl_gobject_properties_{}", object);

    quote!(
                //TODO: GivePropertyBuilderName
                struct #objectbuilder;
                impl #objectbuilder {
                   #pbuilder
                   #sbuilder
                }

                // Will ensure object is a defined type or error;
                impl #object {}

                mod #impl_mod {
                    #![allow(non_snake_case)]
                    fn verify_is_glib_object<T: #glib::IsA<#glib::Object>>(){}
                    fn verify_is_glib_StaticType<T: #glib::StaticType>(){}
    //                fn verify_is_glib_ToValueOptional<T: #glib::value::ToValue>(){}

                    fn test() {
                        verify_is_glib_object::<super::#object>();
                        #pverify
                        #sverify
                    }
                }

                pub trait #objectext {
                    type ThisClass: #glib::IsA<#object>;

                    #pdef
                    #sdef
                }

                impl<T: #glib::IsA<#object>> #objectext for T {
                    type ThisClass = #object;

                    #pimpl
                    #simpl
                }
            )
}

fn handle_gsp_attributes(attr: proc_macro2::TokenStream) {
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
