use gtk4::glib;
use gtk4::glib::subclass::Signal;

use glib::subclass::prelude::*;
#[derive(Default)]
pub struct LoginWidgetImp {}

#[glib::object_subclass]
impl ObjectSubclass for LoginWidgetImp {
    const NAME: &'static str = "LoginWidget";
    type Type = super::MyWidget;
    type ParentType = gtk4::Box;
}

unsafe impl<T: gtk4::subclass::box_::BoxImpl> IsSubclassable<T> for super::MyWidget {
    fn class_init(class: &mut glib::Class<Self>) {
        <gtk4::Box as IsSubclassable<T>>::class_init(class.upcast_ref_mut::<gtk4::Box>());
    }
    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gtk4::Box as IsSubclassable<T>>::instance_init(instance);
    }
}

impl gtk4::subclass::box_::BoxImpl for LoginWidgetImp {}

impl gtk4::subclass::widget::WidgetImpl for LoginWidgetImp {}

impl glib::subclass::object::ObjectImpl for LoginWidgetImp {
    fn properties() -> &'static [glib::ParamSpec] {
        super::MyWidgetObjectSubclassBuilder::properties()
    }

    /// Signals installed for this type.
    fn signals() -> &'static [Signal] {
        super::MyWidgetObjectSubclassBuilder::signals()
    }

    fn get_property(
        &self,
        _obj: &super::MyWidget,
        _id: usize,
        _pspec: &glib::ParamSpec,
    ) -> glib::Value {
        dbg!("Get_property");
        _pspec.name().into()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        _value: &glib::Value,
        _pspec: &glib::ParamSpec,
    ) {
        dbg!("Set_property {:?}", _value);
    }
}
