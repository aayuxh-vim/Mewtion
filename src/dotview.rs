use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

/// A dot to draw: centre x and y, diameter, and alpha - all in widget pixels.
pub type Dot = (f32, f32, f32, f32);

mod imp {
    use super::Dot;
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{gdk, graphene, gsk};
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct DotView {
        pub dots: RefCell<Vec<Dot>>,
        pub color: Cell<(f32, f32, f32)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DotView {
        const NAME: &'static str = "MewtionDotView";
        type Type = super::DotView;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for DotView {}

    impl WidgetImpl for DotView {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let (r, g, b) = self.color.get();
            for &(x, y, size, alpha) in self.dots.borrow().iter() {
                let rect = graphene::Rect::new(x - size / 2.0, y - size / 2.0, size, size);
                let circle = gsk::RoundedRect::from_rect(rect, size / 2.0);
                snapshot.push_rounded_clip(&circle);
                snapshot.append_color(&gdk::RGBA::new(r, g, b, alpha), &rect);
                snapshot.pop();
            }
        }
    }
}

glib::wrapper! {
    pub struct DotView(ObjectSubclass<imp::DotView>)
        @extends gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for DotView {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl DotView {
    /// Submits the dots to draw on the next frame.
    pub fn set_dots(&self, color: (f32, f32, f32), dots: Vec<Dot>) {
        let imp = self.imp();
        imp.color.set(color);
        *imp.dots.borrow_mut() = dots;
        self.queue_draw();
    }
}
