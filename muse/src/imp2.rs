use gtk4::glib;
use gtk4::glib::subclass::Signal;

use glib::subclass::prelude::*;
#[derive(Default)]
pub struct LoginWidgetImp {}

#[glib::object_subclass]
impl ObjectSubclass for LoginWidgetImp {
    const NAME: &'static str = "LoginWidget2";
    type Type = super::MyWidget2;
    type ParentType = super::MyWidget;
}

impl gtk4::subclass::box_::BoxImpl for LoginWidgetImp {}

impl gtk4::subclass::widget::WidgetImpl for LoginWidgetImp {}

impl glib::subclass::object::ObjectImpl for LoginWidgetImp {
    fn properties() -> &'static [glib::ParamSpec] {
        super::MyWidget2ObjectSubclassBuilder::properties()
    }

    /// Signals installed for this type.
    fn signals() -> &'static [Signal] {
        super::MyWidget2ObjectSubclassBuilder::signals()
    }

    fn property(
        &self,
        _obj: &super::MyWidget2,
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
