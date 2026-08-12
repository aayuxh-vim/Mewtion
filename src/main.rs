// motion-dots: Phase 5 -- Detaching Anchors
//
// 10 fixed dots on each side. When stationary, they are locked in place.
// When motion occurs, the fixed dots detach and blow away, while new organic 
// dots spawn. When motion stops, the 10 fixed dots fade back into their slots.

mod tcp;

use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, DrawingArea};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.motiondots.Overlay";

// --- Particle System State ---

struct Rng { state: u32 }
impl Rng {
    fn new() -> Self { Rng { state: 42 } }
    fn next_f64(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        (self.state as f64) / (u32::MAX as f64)
    }
}

struct Particle {
    nx: f64,
    ny: f64,
    life: f64,
    fade_rate: f64,
    size: f64,
    is_anchor: bool, // Helps us treat the 10 fixed dots differently than moving dots
}

struct AppState {
    target_x: f64,
    target_y: f64,
    current_x: f64,
    current_y: f64,
    particles: Vec<Particle>,
    rng: Rng,
}

// --- Main Application ---

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

    window.set_default_size(1920, 1080);

    let css = gtk4::CssProvider::new();
    css.load_from_string("window { background: transparent; }");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("no default display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let state = Rc::new(RefCell::new(AppState {
        target_x: 0.0,
        target_y: 0.0,
        current_x: 0.0,
        current_y: 0.0,
        particles: Vec::new(),
        rng: Rng::new(),
    }));

    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(1920);
    drawing_area.set_content_height(1080);

    {
        let state = state.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let s = state.borrow();
            let w = width as f64;
            let h = height as f64;

            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().ok();

            for p in &s.particles {
                // Anchors use straight opacity. Flow dots use a sine curve to fade in and out.
                let alpha = if p.is_anchor {
                    p.life 
                } else {
                    (p.life * std::f64::consts::PI).sin() * 0.75
                }; 

                cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
                cr.arc(p.nx * w, p.ny * h, p.size, 0.0, std::f64::consts::TAU);
                cr.fill().ok();
            }
        });
    }

    window.set_child(Some(&drawing_area));

    {
        let window_weak = window.downgrade();
        window.connect_realize(move |_| {
            if let Some(window) = window_weak.upgrade() {
                make_overlay_click_through_and_always_on_top(&window);
            }
        });
    }

    window.present();

    let (sender, receiver) = std::sync::mpsc::channel::<tcp::AccelSample>();

    std::thread::spawn(move || {
        tcp::run_tcp_bridge_blocking(move |sample| {
            let _ = sender.send(sample);
        });
    });

    {
        let state_for_loop = state.clone();
        let drawing_area = drawing_area.clone();
        const ACCEL_SENSITIVITY: f32 = 3.0;

 glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Unpack the RefMut smart pointer into a raw mutable reference
            let mut s_guard = state_for_loop.borrow_mut();
            let s = &mut *s_guard;

            while let Ok(sample) = receiver.try_recv() {              s.target_x = (sample.x / ACCEL_SENSITIVITY).clamp(-1.0, 1.0) as f64;
                s.target_y = (sample.y / ACCEL_SENSITIVITY).clamp(-1.0, 1.0) as f64;
            }

            // Lowered to 0.04 for buttery smooth momentum
            s.current_x += (s.target_x - s.current_x) * 0.04;
            s.current_y += (s.target_y - s.current_y) * 0.04;

            let speed = (s.current_x.powi(2) + s.current_y.powi(2)).sqrt();
            let is_moving = speed > 0.025; // Deadzone

            if is_moving {
                // 1. DETACH ANCHORS: The moment we move, unlock the 10 fixed dots.
                for p in &mut s.particles {
                    if p.is_anchor {
                        p.is_anchor = false;
                        
                        // Seamless opacity handoff: sin(0.5 * PI) * 0.75 = 0.75.
                        // This prevents them from flickering when they switch to flow dots.
                        p.life = 0.5; 
                        p.fade_rate = 0.005 + s.rng.next_f64() * 0.01;
                    }
                }

                // 2. SPAWN FLOW DOTS: Keep the screen populated while moving
                let target_count = 35 + (speed * 30.0) as usize;
                if s.particles.len() < target_count {
                    for _ in 0..3 {
                        if s.particles.len() >= target_count { break; }

                        let is_left = s.rng.next_f64() < 0.5;
                        let nx = if is_left { 0.01 + s.rng.next_f64() * 0.05 } else { 0.94 + s.rng.next_f64() * 0.05 };
                        let ny = s.rng.next_f64();
                        
                        let random_fade = 0.005 + s.rng.next_f64() * 0.01;
                        let random_size = 7.0 + s.rng.next_f64() * 3.0; // Bigger moving dots

                        s.particles.push(Particle {
                            nx,
                            ny,
                            life: 1.0, // Starts at 1.0 so the sine wave fades it up from 0
                            fade_rate: random_fade, 
                            size: random_size,
                            is_anchor: false,
                        });
                    }
                }
            } else {
                // 3. RESPAWN ANCHORS: We stopped. Check if our 20 anchors exist.
                let anchor_count = s.particles.iter().filter(|p| p.is_anchor).count();
                
                if anchor_count == 0 {
                    // Populate exactly 10 slots on the left and 10 on the right
                    for i in 0..10 {
                        let ny = (i as f64 + 0.5) / 10.0;
                        
                        // Left anchor
                        s.particles.push(Particle {
                            nx: 0.03, ny, life: 0.0, fade_rate: 0.0, size: 8.0, is_anchor: true,
                        });
                        // Right anchor
                        s.particles.push(Particle {
                            nx: 0.97, ny, life: 0.0, fade_rate: 0.0, size: 8.0, is_anchor: true,
                        });
                    }
                }
            }

            // 4. APPLY PHYSICS
            let cx = s.current_x;
            let cy = s.current_y;
            
            for p in &mut s.particles {
                if p.is_anchor {
                    // Anchors don't move. They just gently fade up to their target opacity (0.75).
                    p.life += (0.75 - p.life) * 0.05;
                } else {
                    // Flow dots drift and die
                    p.nx += cx * 0.01;
                    p.ny += cy * 0.01;
                    p.life -= p.fade_rate;
                }
            }
            
            // Cleanup dead dots
            s.particles.retain(|p| p.life > 0.0 && p.nx > -0.1 && p.nx < 1.1 && p.ny > -0.1 && p.ny < 1.1);

            drawing_area.queue_draw();
            glib::ControlFlow::Continue
        });
    }
}

// --- Window Manager Hints ---

fn make_overlay_click_through_and_always_on_top(window: &ApplicationWindow) {
    use gdk4_x11::X11Surface;
    use x11rb::connection::Connection;
    use x11rb::protocol::shape::{ConnectionExt as ShapeConnExt, SK, SO};
    use x11rb::protocol::xproto::{
        AtomEnum, ChangeWindowAttributesAux, ClipOrdering, ConnectionExt, PropMode,
    };
    use x11rb::wrapper::ConnectionExt as _; 

    let Some(surface) = window.surface() else { return };
    let Some(x11_surface) = surface.downcast_ref::<X11Surface>() else { return };

    let xid = x11_surface.xid();
    let Ok((conn, screen_num)) = x11rb::connect(None) else { return };
    let screen = &conn.setup().roots[screen_num];
    let win = xid as u32;

    let aux = ChangeWindowAttributesAux::new().override_redirect(1u32);
    let _ = conn.change_window_attributes(win, &aux);

    if let (Ok(state_atom), Ok(above_atom)) = (intern(&conn, "_NET_WM_STATE"), intern(&conn, "_NET_WM_STATE_ABOVE")) {
        let _ = conn.change_property32(PropMode::REPLACE, win, state_atom, AtomEnum::ATOM, &[above_atom]);
    }

    if let (Ok(type_atom), Ok(utility_atom)) = (intern(&conn, "_NET_WM_WINDOW_TYPE"), intern(&conn, "_NET_WM_WINDOW_TYPE_UTILITY")) {
        let _ = conn.change_property32(PropMode::REPLACE, win, type_atom, AtomEnum::ATOM, &[utility_atom]);
    }

    let _ = conn.shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED, win, 0, 0, &[]);
    let _ = conn.flush();
    let _ = screen; 
}

fn intern(conn: &impl x11rb::connection::Connection, name: &str) -> Result<x11rb::protocol::xproto::Atom, ()> {
    use x11rb::protocol::xproto::ConnectionExt;
    conn.intern_atom(false, name.as_bytes()).map_err(|_| ())?.reply().map(|r| r.atom).map_err(|_| ())
}
