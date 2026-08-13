
// Mewtion: Phase 7 -- Native Wayland (Layer Shell)
//
// Completely drops X11 legacy hacks. Uses native Wayland protocols to create
// an always-on-top overlay. By setting an empty cairo input region on the 
// surface, Wayland cleanly passes all mouse clicks to the desktop below.

mod tcp;

use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, DrawingArea};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.mewtion.Overlay";

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
    is_anchor: bool,
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
        .title("Mewtion")
        .build();

    // 1. Initialize Wayland Layer Shell
    window.init_layer_shell();
    
    // Set to Overlay layer (Always on top of standard windows)
    window.set_layer(Layer::Overlay);
    
    // Don't push other windows out of the way
    window.set_exclusive_zone(-1); 
    
    // Stretch to fill the entire screen
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // 2. Make it transparent
    let css = gtk4::CssProvider::new();
    css.load_from_data("window { background: transparent; }");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("no default display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // 3. The Click-Through Magic (Native Wayland)
    window.connect_realize(|w| {
        if let Some(surface) = w.surface() {
            // Create a 0x0 hit-box region. Wayland will instantly pass all 
            // mouse clicks through this window to whatever is behind it!
            let empty_region = gtk4::cairo::Region::create();
            surface.set_input_region(Some(&empty_region));
        }
    });

    let state = Rc::new(RefCell::new(AppState {
        target_x: 0.0,
        target_y: 0.0,
        current_x: 0.0,
        current_y: 0.0,
        particles: Vec::new(),
        rng: Rng::new(),
    }));

    let drawing_area = DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    {
        let state = state.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let s = state.borrow();
            let w = width as f64;
            let h = height as f64;

            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().ok();

            for p in &s.particles {
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
    window.present();

    let (sender, receiver) = std::sync::mpsc::channel::<tcp::AccelSample>();

    std::thread::spawn(move || {
        tcp::run_tcp_bridge_blocking(move |sample| {
            let _ = sender.send(sample);
        });
    });

    let state_for_loop = state.clone();
    let drawing_area = drawing_area.clone();
    const ACCEL_SENSITIVITY: f32 = 3.0;

    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let mut s_guard = state_for_loop.borrow_mut();
        let s = &mut *s_guard;

        while let Ok(sample) = receiver.try_recv() {
            s.target_x = (sample.x / ACCEL_SENSITIVITY).clamp(-1.0, 1.0) as f64;
            s.target_y = (sample.y / ACCEL_SENSITIVITY).clamp(-1.0, 1.0) as f64;
        }

        s.current_x += (s.target_x - s.current_x) * 0.04;
        s.current_y += (s.target_y - s.current_y) * 0.04;

        let speed = (s.current_x.powi(2) + s.current_y.powi(2)).sqrt();
        let is_moving = speed > 0.025; 

        if is_moving {
            for p in &mut s.particles {
                if p.is_anchor {
                    p.is_anchor = false;
                    p.life = 0.5; 
                    p.fade_rate = 0.005 + s.rng.next_f64() * 0.01;
                }
            }

            let target_count = 35 + (speed * 30.0) as usize;
            if s.particles.len() < target_count {
                for _ in 0..3 {
                    if s.particles.len() >= target_count { break; }

                    let is_left = s.rng.next_f64() < 0.5;
                    let nx = if is_left { 0.01 + s.rng.next_f64() * 0.05 } else { 0.94 + s.rng.next_f64() * 0.05 };
                    let ny = s.rng.next_f64();
                    
                    let random_fade = 0.005 + s.rng.next_f64() * 0.01;
                    let random_size = 7.0 + s.rng.next_f64() * 3.0; 

                    s.particles.push(Particle {
                        nx, ny, life: 1.0, fade_rate: random_fade, size: random_size, is_anchor: false,
                    });
                }
            }
        } else {
            let anchor_count = s.particles.iter().filter(|p| p.is_anchor).count();
            if anchor_count == 0 {
                for i in 0..10 {
                    let ny = (i as f64 + 0.5) / 10.0;
                    s.particles.push(Particle { nx: 0.03, ny, life: 0.0, fade_rate: 0.0, size: 8.0, is_anchor: true });
                    s.particles.push(Particle { nx: 0.97, ny, life: 0.0, fade_rate: 0.0, size: 8.0, is_anchor: true });
                }
            }
        }

        let cx = s.current_x;
        let cy = s.current_y;
        
        for p in &mut s.particles {
            if p.is_anchor {
                p.life += (0.75 - p.life) * 0.05;
            } else {
                p.nx += cx * 0.01;
                p.ny += cy * 0.01;
                p.life -= p.fade_rate;
            }
        }
        
        s.particles.retain(|p| p.life > 0.0 && p.nx > -0.1 && p.nx < 1.1 && p.ny > -0.1 && p.ny < 1.1);
        drawing_area.queue_draw();

        glib::ControlFlow::Continue
    });
}
