use egui;
use egui_glow::glow;
use egui_demo_lib::DemoWindows;
use fltk::{app, *, prelude::*, window::GlutWindow};
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(
        100,
        100,
        SCREEN_WIDTH as _,
        SCREEN_HEIGHT as _,
        Some("Demo window"),
    )
    .center_screen();
    win.set_mode(enums::Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let demo = egui_demo_lib::DemoWindows::default();
    run_egui(fltk_app, win, demo);
}

fn run_egui(fltk_app: app::App, mut win: GlutWindow, demo: DemoWindows) {
    // Init backend
    let (mut painter, egui_state) = fltk_egui::init(&mut win);
    let state = Rc::new(RefCell::new(egui_state));

    win.handle({
        let state = state.clone();
        move |win, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag => {
                // Using "if let ..." for safety.
                if let Ok(mut state) = state.try_borrow_mut() {
                    state.fuse_input(win, ev);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    let egui_ctx = egui::Context::default();
    let start_time = Instant::now();
    let mut demo_windows = demo;

    while fltk_app.wait() {
        // Clear the screen to dark red
        let gl = painter.gl().as_ref();
        draw_background(gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            demo_windows.ui(&ctx);
        });

        if egui_ctx.has_requested_repaint() || state.window_resized() {
            //Draw egui texture
            state.fuse_output(&mut win, egui_output.platform_output);
            let meshes = egui_ctx.tessellate(egui_output.shapes, win.pixels_per_unit());
            painter.paint_and_update_textures(
                state.canvas_size,
                state.pixels_per_point(),
                &meshes,
                &egui_output.textures_delta,
            );
            win.swap_buffers();
            win.flush();
            app::awake();
        }
    }

    painter.destroy();
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.clear(glow::DEPTH_BUFFER_BIT);
    }
}
