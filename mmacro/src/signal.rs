#![allow(dead_code, unreachable_code)]

pub(crate) struct Signal {
    name: crate::ParamSpecName,
    output: syn::ReturnType

}

pub(crate) fn handle_signal(tim: syn::TraitItemMethod) -> Signal {
    #[derive(Default)]
    struct SignalAttributes {
        magic: bool,
        class_handler_flags: bool,
    }
    
    let mut attributes = SignalAttributes::default();
    for a in &tim.attrs {
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
            },
            // #[class_handler(run_first, run_last, run_cleanup)]
            "class_handler" => {
                attributes.class_handler_flags = true;
            } 
            _ => proc_macro_error::abort!(a.path, "gobject_signal_properties unexpected")

        }
//        println!("{:?}", attribute_name)
    }


    if !attributes.magic {
        proc_macro_error::abort!(tim, "gobject_signal_properties: missing signal attribute")
    }

    if attributes.class_handler_flags {
        proc_macro_error::abort!(tim, "gobject_signal_properties: default class handler unimplemented")
    }

    if let Some(b) = tim.default {
        proc_macro_error::abort!(b, "gobject_signal_properties: default class handler unimplemented")
    }


    if let Some(c) = tim.sig.constness {
        proc_macro_error::abort!(c, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = tim.sig.asyncness {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = tim.sig.unsafety {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    if let Some(a) = tim.sig.abi {
        proc_macro_error::abort!(a, "gobject_signal_properties: unsupported")
    }

    let name = crate::ParamSpecName::new(&tim.sig.ident).unwrap_or_else(|()|proc_macro_error::abort!(tim.sig.ident, "gobject_signal_properties: invalid name"));


    if let Some(_) = tim.sig.generics.lt_token {
        proc_macro_error::abort!(tim.sig.generics.params, "gobject_signal_properties unexpected")        
    }

    if let Some(w) = tim.sig.generics.where_clause {
        proc_macro_error::abort!(w, "gobject_signal_properties unexpected")        
    }

    // tim.sig.inputs

    if let Some(v) = tim.sig.variadic {
        proc_macro_error::abort!(v, "gobject_signal_properties unsupported")        
    }

    Signal {
        name,

        output: tim.sig.output
    }
}
