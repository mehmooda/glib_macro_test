use mmacro::gobject_signal_properties;

mod imp;
mod imp2;

gtk4::glib::wrapper! {
    pub struct MyWidget(ObjectSubclass<imp::LoginWidgetImp>)
    @extends gtk4::Widget, gtk4::Box;
}

gtk4::glib::wrapper! {
    pub struct MyWidget2(ObjectSubclass<imp2::LoginWidgetImp>)
    @extends MyWidget, gtk4::Widget, gtk4::Box;
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

#[gobject_signal_properties]
trait MyWidget2 {
    #[signal]
    fn my_signal2(&self, arg1: i64, arg2: u32, arg3: Option<gtk4::Box>) -> gtk4::Box;
    //    TODO: {  default class handler }
    #[property]
    //    TODO: #[nick("A")]
    //    TODO: #[blurb("B")]
    //    TODO: #[flags(READWRITE)]
    type my_other_property = String;

    #[property]
    type another_other_property = String;
}

fn test() -> impl FnMut(u8) {
    let mut x = 0;
    move |n| {
        x += n;
    }
}

fn main() {
    gtk4::init().unwrap();

    let x: MyWidget2 = gtk4::glib::Object::new::<MyWidget2>(&[]).unwrap();
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

    let y = x.get_my_other_property();
    let z = x.get_another_other_property();

    dbg!(y);
    dbg!(z);
}
