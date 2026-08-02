










// motion-dots: Phase 1 prototype
//
// A transparent, always-on-top, click-through overlay that draws dots
// drifting at the screen edges, driven (for now) by a fake sine wave
// instead of real accelerometer data.
//
// Runs via XWayland override-redirect, so it works the same way whether
// the underlying compositor is Hyprland, KWin, or Mutter -- as long as
// XWayland is available (it is, by default, on all three).
//
// NOTE: this must be run on a real desktop session with a display server.
// It will not do anything useful in a headless container.

use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, DrawingArea};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

const APP_ID: &str = "dev.motiondots.Overlay";
const DOT_COUNT: usize = 24;
const DOT_RADIUS: f64 = 4.0;

/// Fake motion state. Later this gets replaced by real accel data
/// (either from /sys/bus/iio or from the phone-over-BLE bridge).
/// Keeping this as a plain (x, y) drift offset in [-1.0, 1.0] range
/// means the render code never needs to know where the data came from.
#[derive(Clone, Copy, Default)]
struct MotionSample {
    x: f64,
    y: f64,
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("motion-dots")
        .decorated(false)
        .resizable(false)
        .build();

    // Full-screen-sized transparent canvas. We don't know the real
    // monitor size in this generic build step, so default to something
    // reasonable; swap for gdk4::Monitor geometry once you wire up
    // multi-monitor support.
    window.set_default_size(1920, 1080);

    // Transparent background via CSS -- GTK4 windows are already capable
    // of alpha compositing as long as the compositor supports it, which
    // all three targets do.
    let css = gtk4::CssProvider::new();
    css.load_from_data("window { background: transparent; }");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("no default display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let motion = Rc::new(RefCell::new(MotionSample::default()));
    let start = Instant::now();

    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(1920);
    drawing_area.set_content_height(1080);

    {
        let motion = motion.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let m = *motion.borrow();
            let w = width as f64;
            let h = height as f64;

            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().ok();

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.55);

            // Dots ringed around the edges, each offset by the current
            // motion sample -- mirrors the "drift opposite to motion"
            // behavior Apple's Vehicle Motion Cues uses.
            for i in 0..DOT_COUNT {
                let t = i as f64 / DOT_COUNT as f64;
                let angle = t * std::f64::consts::TAU;
                let edge_x = w / 2.0 + (w / 2.0 - 20.0) * angle.cos();
                let edge_y = h / 2.0 + (h / 2.0 - 20.0) * angle.sin();

                let dx = edge_x + m.x * 30.0;
                let dy = edge_y + m.y * 30.0;

                cr.arc(dx, dy, DOT_RADIUS, 0.0, std::f64::consts::TAU);
                cr.fill().ok();
            }
        });
    }

    window.set_child(Some(&drawing_area));

    // Real X11 setup happens once the underlying surface exists.
    {
        let window_weak = window.downgrade();
        window.connect_realize(move |_| {
            if let Some(window) = window_weak.upgrade() {
                make_overlay_click_through_and_always_on_top(&window);
            }
        });
    }

    window.present();

    // Fake sine-wave motion source, ~60fps. Swap this closure for a
    // real sensor read (IIO poll or BLE packet handler) later --
    // nothing else in this file needs to change.
    {
        let motion = motion.clone();
        let drawing_area = drawing_area.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let t = start.elapsed().as_secs_f64();
            let mut m = motion.borrow_mut();
            m.x = (t * 0.8).sin();
            m.y = (t * 0.5).cos() * 0.6;
            drop(m);
            drawing_area.queue_draw();
            glib::ControlFlow::Continue
        });
    }
}

/// Make the window an override-redirect, always-on-top, click-through
/// overlay via raw X11 (through XWayland). This is the part that lets
/// the same binary behave correctly whether it's actually running under
/// Hyprland, KWin, or Mutter -- they all proxy XWayland clients the same
/// way, so we never touch a compositor-specific Wayland protocol here.
fn make_overlay_click_through_and_always_on_top(window: &ApplicationWindow) {
    use gdk4_x11::X11Surface;
    use x11rb::connection::Connection;
    use x11rb::protocol::shape::{ConnectionExt as ShapeConnExt, SK, SO};
    use x11rb::protocol::xproto::{
        AtomEnum, ChangeWindowAttributesAux, ClipOrdering, ConnectionExt, PropMode,
    };
    use x11rb::wrapper::ConnectionExt as _; // brings change_property32 into scope

    let Some(surface) = window.surface() else { return };
    let Some(x11_surface) = surface.downcast_ref::<X11Surface>() else {
        eprintln!(
            "motion-dots: not running under XWayland/X11, skipping overlay setup \
             (window will still show, but won't be click-through or always-on-top)"
        );
        return;
    };

    let xid = x11_surface.xid();

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        eprintln!("motion-dots: failed to open raw X11 connection");
        return;
    };
    let screen = &conn.setup().roots[screen_num];
    let win = xid as u32;

    // 1. Override-redirect: tells the WM to leave this window alone
    //    entirely (no decorations, no focus stealing, no taskbar entry).
    let aux = ChangeWindowAttributesAux::new().override_redirect(1u32);
    let r: Result<(), Box<dyn std::error::Error>> = (|| {
        conn.change_window_attributes(win, &aux)?.check()?;
        Ok(())
    })();
    eprintln!("motion-dots: override_redirect: {r:?}");

    // 2. _NET_WM_STATE_ABOVE: keep it above normal windows.
    if let (Ok(state_atom), Ok(above_atom)) = (
        intern(&conn, "_NET_WM_STATE"),
        intern(&conn, "_NET_WM_STATE_ABOVE"),
    ) {
        let r: Result<(), Box<dyn std::error::Error>> = (|| {
            conn.change_property32(
                PropMode::REPLACE,
                win,
                state_atom,
                AtomEnum::ATOM,
                &[above_atom],
            )?
            .check()?;
            Ok(())
        })();
        eprintln!("motion-dots: _NET_WM_STATE_ABOVE: {r:?}");
    }

    // 3. _NET_WM_WINDOW_TYPE_UTILITY: hints WMs to treat it as a
    //    non-interactive utility overlay rather than a normal window.
    if let (Ok(type_atom), Ok(utility_atom)) = (
        intern(&conn, "_NET_WM_WINDOW_TYPE"),
        intern(&conn, "_NET_WM_WINDOW_TYPE_UTILITY"),
    ) {
        let r: Result<(), Box<dyn std::error::Error>> = (|| {
            conn.change_property32(
                PropMode::REPLACE,
                win,
                type_atom,
                AtomEnum::ATOM,
                &[utility_atom],
            )?
            .check()?;
            Ok(())
        })();
        eprintln!("motion-dots: _NET_WM_WINDOW_TYPE: {r:?}");
    }

    // 4. Click-through: set an empty *input* shape region, so all
    //    clicks/scrolls pass straight through to whatever is underneath.
    //    The window still *renders* normally -- only input is affected.
    let r: Result<(), Box<dyn std::error::Error>> = (|| {
        conn.shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED, win, 0, 0, &[])?
            .check()?;
        Ok(())
    })();
    eprintln!("motion-dots: shape_rectangles (click-through): {r:?}");

    let _ = conn.flush();
    let _ = screen; // silence unused warning if you strip pieces above
}

fn intern(
    conn: &impl x11rb::connection::Connection,
    name: &str,
) -> Result<x11rb::protocol::xproto::Atom, ()> {
    use x11rb::protocol::xproto::ConnectionExt;
    conn.intern_atom(false, name.as_bytes())
        .map_err(|_| ())?
        .reply()
        .map(|r| r.atom)
        .map_err(|_| ())
}
