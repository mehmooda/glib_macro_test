use gtk4::{glib::subclass::Signal};
use gtk4::glib;

use glib::subclass::prelude::*;
use glib::once_cell::sync::OnceCell;
#[derive(Default)]
pub struct LoginWidgetImp {}

#[glib::object_subclass]
impl ObjectSubclass for LoginWidgetImp {
    const NAME: &'static str = "LoginWidget";
    type Type = super::MyWidget;
    type ParentType = gtk4::Box;
}

impl gtk4::subclass::box_::BoxImpl for LoginWidgetImp {}

impl gtk4::subclass::widget::WidgetImpl for LoginWidgetImp {}

impl glib::subclass::object::ObjectImpl for LoginWidgetImp {
    fn properties() -> &'static [glib::ParamSpec] {
        super::_todo_change_name_property_build()
    }

    /// Signals installed for this type.
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceCell<Vec<Signal>> = OnceCell::new();

        SIGNALS.get_or_init(|| {
            vec![]
        })
    }

    fn get_property(
        &self,
        _obj: &super::MyWidget,
        _id: usize,
        _pspec: &glib::ParamSpec,
    ) -> glib::Value {
        dbg!("Get_property");
        _pspec.get_name().into()
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