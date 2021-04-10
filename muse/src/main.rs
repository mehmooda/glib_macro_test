use mmacro::gobject_signal_properties;

mod imp;

gtk4::glib::wrapper! {
    pub struct MyWidget(ObjectSubclass<imp::LoginWidgetImp>)
    @extends gtk4::Box, gtk4::Widget;
}

#[gobject_signal_properties]
trait MyWidget {
    #[signal]
    fn my_signal(&self);
    //    TODO: {  default class handler }
    #[property]
    //    TODO: #[nick("A")]
    //    TODO: #[blurb("B")]
    //    TODO: #[flags(READWRITE)]
    type my_property = String;

    #[property]
    type another_property = String;
}

fn main() {
    gtk4::init();

    let x = gtk4::glib::Object::new::<MyWidget>(&[]).unwrap();
    let y = x.get_my_property();
    let z = x.get_another_property();

    dbg!(y);
    dbg!(z);

    //TODO:
    //x.connect_my_signal();
}
