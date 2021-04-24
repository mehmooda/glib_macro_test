#![allow(dead_code, unreachable_code)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ReturnType};

use crate::TurnOptionIntoInner;

pub(crate) struct Signal {
    name: crate::ParamSpecName,
    inputs: Vec<FnArg>,
    output: syn::Type,
    default: Option<syn::Block>,
}

pub(crate) fn handle_signal(tim: &syn::TraitItemMethod) -> Signal {
    let attributes = parse_signal_attributes(&tim.attrs);

    if !attributes.magic {
        proc_macro_error::abort!(
            tim.sig,
            "gobject_signal_properties: Missing signal attribute"
        )
    }

    let default = tim.default.clone();

    if let Some(b) = &tim.default {
        proc_macro_error::abort!(
            b,
            "gobject_signal_properties: default class handler unimplemented"
        )
    }

    let (name, inputs, output) = parse_signal_definition(&tim.sig);

    Signal {
        name,
        inputs,
        output,
        default,
    }
}

#[derive(Default)]
struct SignalAttributes {
    magic: bool,
    class_handler_flags: bool,
}

fn parse_signal_attributes(attrs: &[syn::Attribute]) -> SignalAttributes {
    let mut attributes = SignalAttributes::default();

    for a in attrs {
        if let Some(x) = a.path.leading_colon {
            proc_macro_error::abort!(x, "gobject_signal_properties unexpected")
        }
        if a.path.segments.len() != 1 {
            proc_macro_error::abort!(a.path, "gobject_signal_properties unexpected")
        }
        let attribute_name = &a.path.segments[0].ident;
        match format!("{}", attribute_name).as_str() {
            "signal" => {
                if !a.tokens.is_empty() {
                    proc_macro_error::abort!(a.tokens, "gobject_signal_properties unexpected")
                }
                attributes.magic = true;
            }
            // #[class_handler(run_first, run_last, run_cleanup)]
            "class_handler" => {
                proc_macro_error::abort!(
                    a,
                    "gobject_signal_properties: default class handler unimplemented"
                );

                //                attributes.class_handler_flags = true;
            }
            _ => proc_macro_error::abort!(a.path, "gobject_signal_properties unexpected"),
        }
        //        println!("{:?}", attribute_name)
    }

    attributes
}

fn parse_signal_definition(sig: &syn::Signature) -> (super::ParamSpecName, Vec<FnArg>, syn::Type) {
    if let Some(c) = sig.constness {
        proc_macro_error::abort!(c, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = sig.asyncness {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = sig.unsafety {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = &sig.abi {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    let name = crate::ParamSpecName::new(&sig.ident).unwrap_or_else(|()| {
        proc_macro_error::abort!(sig.ident, "gobject_signal_properties: invalid name")
    });

    if let Some(_) = sig.generics.lt_token {
        proc_macro_error::abort!(sig.generics.params, "gobject_signal_properties unexpected")
    }

    if let Some(w) = &sig.generics.where_clause {
        proc_macro_error::abort!(w, "gobject_signal_properties unexpected")
    }

    if sig.inputs.len() == 0 {
        proc_macro_error::abort!(
            sig,
            "gobject_signal_properties signal first argument must be &self"
        )
    }

    match &sig.inputs[0] {
        syn::FnArg::Typed(x) => proc_macro_error::abort!(
            x,
            "gobject_signal_properties signal first argument must be &self"
        ),
        syn::FnArg::Receiver(receiver) => {
            if !receiver.attrs.is_empty() {
                proc_macro_error::abort!(
                    receiver,
                    "gobject_signal_properties no attributes allowed"
                )
            }
            if receiver.reference.is_none() || receiver.mutability.is_some() {
                proc_macro_error::abort!(
                    receiver,
                    "gobject_signal_properties first argument must be &self"
                )
            }
        }
    }

    let types = sig.inputs.iter().skip(1).cloned().collect::<Vec<_>>();

    if let Some(v) = &sig.variadic {
        proc_macro_error::abort!(v, "gobject_signal_properties unsupported")
    }

    let output = if let ReturnType::Type(_, x) = &sig.output {
        *x.clone()
    } else {
        syn::parse2(quote!(())).unwrap()
    };

    (name, types, output)
}

pub(crate) fn verifications(signals: &[Signal]) -> TokenStream {
    let verify = signals
        .iter()
        .map(|s| {
            let output = s.output.inner_if_option();
            let inputs = s.inputs.iter().map(|f| {
                if let FnArg::Typed(x) = f {
                    x.ty.inner_if_option()
                } else {
                    proc_macro_error::abort!(f, "gobject_signal_properties expected type")
                }
            });
            quote! {
                verify_is_glib_StaticType::<#output>();
                #(
                    verify_is_glib_StaticType::<#inputs>();
                )*
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(
            #verify
        )*
    }
}

pub(crate) fn definitions(signals: &[Signal]) -> TokenStream {
    let connect_signal = signals
        .iter()
        .map(|s| quote::format_ident!("connect_{}", s.name.snake_case()));
    let emit_signal = signals
        .iter()
        .map(|s| quote::format_ident!("emit_{}", s.name.snake_case()));

    let args = signals.iter().map(|s| &s.inputs);
    let arg_types = signals.iter().map(|s| {
        s.inputs
            .iter()
            .map(|i| match i {
                FnArg::Typed(p) => p.ty.clone(),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
    });

    let out = signals.iter().map(|s| &s.output);
    quote! {
        #(
            fn #connect_signal<F: Fn(&Self, #(#arg_types),*) -> #out + 'static>(&self, callback: F);
            fn #emit_signal(&self, #(#args),*) -> #out;
        )*
    }
}

fn convert_ret(signal: &Signal) -> TokenStream {
    if signal.output == syn::parse2(quote!(())).unwrap() {
        return quote!(None);
    }
    if signal.output.is_option() {
        return quote! {
        //            None
                    ret.map(|z| z.to_value())
                };
    }
    return quote! {
        Some(ret.to_value())
    };
}

fn handle_emit_option(signal: &Signal) -> TokenStream {
    if signal.output == syn::parse2(quote!(())).unwrap() {
        return quote!(.map(|o| panic!("Expected Unit found {}", o.type_())););
    }
    if signal.output.is_option() {
        return quote! {
            .map(|z| z.get().unwrap().unwrap())
        };
    }
    return quote! {
        .unwrap().get().unwrap().unwrap()
    };
}

pub(crate) fn implementations(signals: &[Signal]) -> TokenStream {
    let glib = super::get_glib();

    let signal = signals.iter().map(|s| s.name.kebab_case());
    let connect_signal = signals
        .iter()
        .map(|s| quote::format_ident!("connect_{}", s.name.snake_case()));
    let emit_signal = signals
        .iter()
        .map(|s| quote::format_ident!("emit_{}", s.name.snake_case()));
    let args = signals.iter().map(|s| &s.inputs);
    let arg_types = signals
        .iter()
        .map(|s| {
            s.inputs
                .iter()
                .map(|i| match i {
                    FnArg::Typed(p) => p.ty.clone(),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

//    let inner_arg_types = arg_types
//        .iter()
//        .map(|x| x.iter().map(|z| z.inner_if_option()).collect::<Vec<_>>());
    let unwrap_if_inner = arg_types.iter().map(|z| {
        z.iter()
            .map(|t| !t.is_option())
            .map(|t| {
                if t {
                    quote! {.unwrap()}
                } else {
                    quote! {}
                }
            })
            .collect::<Vec<_>>()
    });
    let numbers = (0..signals.len()).map(|z| {
        (0..signals[z].inputs.len())
            .map(syn::Index::from)
            .collect::<Vec<_>>()
    });
    let out = signals.iter().map(|s| &s.output);

    let get_name = |t: &FnArg| {
        if let FnArg::Typed(p) = &t {
            if let syn::Pat::Ident(pi) = &*p.pat {
                return pi.ident.clone();
            }
        }
        proc_macro_error::abort_call_site!("Expected ident in fnarg")
    };

    let arg_names = signals
        .iter()
        .map(|s| s.inputs.iter().map(get_name).collect::<Vec<_>>());

    let convert_ret = signals.iter().map(convert_ret);
    let handle_emit_option = signals.iter().map(handle_emit_option);

    quote! {
        #(
            fn #connect_signal<F: Fn(&Self, #(#arg_types),*) -> #out + 'static>(&self, callback: F){
                use #glib ::value::ToValue;
                use #glib ::ObjectExt;
                use #glib ::Cast;
                
                self.as_ref().connect_local(#signal, false, move |args| {
                    let converted_arg = (
                        #(
                            args[#numbers + 1].get::<#arg_types>().unwrap()
                        ),*
                    ,);
                    
                    let obj = args[0].get::<Self::ThisClass>().unwrap().downcast().unwrap();

                    let ret = callback(&obj,#(converted_arg. #numbers),*);
                    #convert_ret
                }).unwrap();
            }
            fn #emit_signal(&self, #(#args),*) -> #out {
                use #glib ::value::ToValue;
                use #glib ::ObjectExt;
                self.as_ref().emit_by_name_with_values(
                    #signal,
                    &[#(
                        #arg_names.to_value()
                    ),*]
                ).unwrap().unwrap().get().unwrap()
            }
        )*
    }
}

fn class_handler(signal: &Signal) -> TokenStream {
    if signal.default.is_none() {
        return quote! {};
    }

    let _x = signal.default.as_ref().unwrap();

    quote! {
        .class_handler(
            ||
        )
    }
}

pub(crate) fn builder(signals: &[Signal]) -> TokenStream {
    let glib = super::get_glib();
    // TODO: Parse property type and call correct paramspec builder for type
    let signal_name = signals.iter().map(|x| x.name.kebab_case());

    let arg_types = signals.iter().map(|s| {
        s.inputs
            .iter()
            .map(|i| match i {
                FnArg::Typed(p) => p.ty.inner_if_option().clone(),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
    });

    let out = signals.iter().map(|s| s.output.inner_if_option());

    let class_handler = signals.iter().map(class_handler);

    quote! {
        fn signals() -> &'static [#glib ::subclass::Signal] {
            static SIGNALS: #glib ::once_cell::sync::OnceCell<Vec<#glib ::subclass::Signal>> = #glib ::once_cell::sync::OnceCell::new();

            SIGNALS.get_or_init( || vec![
            #(
                #glib ::subclass::Signal::builder(#signal_name,
                    &[
                        #(
                        <#arg_types as #glib ::types::StaticType>::static_type().into()
                        ),*
                    ],
                    <#out as #glib ::types::StaticType>::static_type().into()
                )

                #class_handler

                .build()
            ),*])
        }
    }
}
