use mmacro::gobject_signal_properties;

mod imp;

gtk4::glib::wrapper! {
    pub struct MyWidget(ObjectSubclass<imp::LoginWidgetImp>);
}

#[gobject_signal_properties]
trait MyWidget {
    #[signal]
    fn my_signal(&self, arg1: i64, arg2: u32, arg3: Option<gtk4::Box>) -> gtk4::Box;
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
    gtk4::init().unwrap();

    let x: MyWidget = gtk4::glib::Object::new::<MyWidget>(&[]).unwrap();
    let y = x.get_my_property();
    let z = x.get_another_property();

    dbg!(y);
    dbg!(z);

    x.connect_my_signal(|s, a1, a2, a3| {
        dbg!("called");
        dbg!(s);
        dbg!(a1);
        dbg!(a2);
        dbg!(a3);

        s.clone().upcast()
    });

    use gtk4::glib::object::Cast;

    let ret = x.emit_my_signal(1, 2, Some(x.clone().upcast()));
    dbg!(ret);
}
