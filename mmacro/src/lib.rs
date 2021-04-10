use proc_macro::TokenStream;

mod property;
mod signal;

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn gobject_signal_properties(attr: TokenStream, _item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        proc_macro_error::abort_call_site!(
            "gobject_signal_properties does not support extra arguments"
        );
    }

    let parsed: Result<syn::ItemTrait, _> = syn::parse(_item);
    if let Err(y) = parsed {
        return y.into_compile_error().into();
    }

    let item_trait = parsed.unwrap();
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

    if let Some(w) = item_trait.generics.where_clause {
        proc_macro_error::abort!(w, "gobject_signal_properties unexpected")
    }
    if let Some(_) = item_trait.colon_token {
        proc_macro_error::abort!(
            item_trait.supertraits,
            "gobject_signal_properties unexpected"
        )
    }

    let mut signals = vec![];
    let mut properties = vec![];

    for x in item_trait.items {
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

    let glib = get_glib();

    let property_type = properties.iter().map(|x| &x.object);
    let property_type_verify = quote::quote!{
        #(
            verify_is_glib_FromValueOptional::<#property_type>();
            verify_is_glib_ToValueOptional::<#property_type>();
        )*
    };
    let property_type = properties.iter().map(|x| &x.object);
    let property_func_name = properties.iter().map(|x| syn::Ident::new(&format!("get_{}",&x.name.snake_case()), x.span));
    let property_getters_def = quote::quote! {
        #(
            fn #property_func_name (&self) -> #property_type;
        )*
    };

    let property_type = properties.iter().map(|x| &x.object);
    let property_func_name = properties.iter().map(|x| syn::Ident::new(&format!("get_{}",&x.name.snake_case()), x.span));
    let property_property_name = properties.iter().map(|x| x.name.kebab_case());
    let property_getters_impl = quote::quote! {
        #(
            fn #property_func_name (&self) -> #property_type {
               #glib ::ObjectExt::get_property(self, #property_property_name ).unwrap().get().unwrap().unwrap()
            }
        )*
    };

    // TODO: Parse property type and call correct paramspec builder for type
    let property_property_name = properties.iter().map(|x| x.name.kebab_case());
    let property_builder = quote::quote!{
        fn _todo_change_name_property_build() -> &'static [#glib ::ParamSpec] {
            static PROPERTIES: #glib ::once_cell::sync::OnceCell<Vec<#glib ::ParamSpec>> = #glib ::once_cell::sync::OnceCell::new();
        

            PROPERTIES.get_or_init( || vec![
            #(
                #glib ::ParamSpec::string(#property_property_name,#property_property_name,#property_property_name, None, #glib ::ParamFlags::READWRITE)

            ),*])
        } 
    };




    let widget = item_trait.ident;
    let widgetext = quote::format_ident!("{}Ext", widget);


    let impl_mod = quote::format_ident!("__impl_gobject_properties_{}", widget);

    quote::quote!(

        //TODO: GivePropertyBuilderName
        #property_builder


        // Will ensure object is a defined type or error;
        impl #widget {}

        mod #impl_mod {
            #![allow(non_snake_case)]
            fn verify_is_glib_object<T: #glib ::IsA< #glib ::Object>>(){}
            fn verify_is_glib_FromValueOptional<'a, T: #glib ::value::FromValueOptional<'a>>(){}
            fn verify_is_glib_ToValueOptional<T: #glib ::value::ToValue>(){}

            fn test() {
                verify_is_glib_object::<super::#widget>();
                #property_type_verify
            }
        }



        trait #widgetext {
            #property_getters_def
        }

        impl<T: #glib::IsA<#widget>> #widgetext for T {
            #property_getters_impl
        }



    )
    .into()


}


fn get_glib() -> proc_macro2::TokenStream {
    use proc_macro_crate::*;

    let found_crate = crate_name("glib");
    if let Ok(s) = found_crate {
        let s = syn::Ident::new(&s, proc_macro2::Span::call_site());
        return quote::quote!( #s );
    }

    let found_crate = crate_name("gtk4");
    if let Ok(s) = found_crate 
    {
        let s = syn::Ident::new(&s, proc_macro2::Span::call_site());
        return quote::quote!( ::#s::glib );
    }
    
    panic!("Unable to find glib")

}


struct ParamSpecName(String);

#[allow(dead_code, unreachable_code)]
impl ParamSpecName {
    fn new(name: &syn::Ident) -> Result<Self, ()> {
        let name = format!("{}", name);

        if !name.as_bytes().iter().enumerate().all(|(i, c)| {
            i != 0 && (*c >= b'0' && *c <= b'9' || *c == b'_')
                || (*c >= b'A' && *c <= b'Z')
                || (*c >= b'a' && *c <= b'z')
        }){
            return Err(())
        }

        Ok(ParamSpecName(name))
    }
    fn snake_case(&self) -> &str {
        &self.0
    }
    fn kebab_case(&self) -> String {
        self.0.replace("_", "-")
    }
}
