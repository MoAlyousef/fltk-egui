use egui_backend::{
    egui::{self, Label},
    egui_glow::glow,
    fltk::{prelude::*, *},
    EguiImageConvertible, EguiSvgConvertible,
};
use fltk::{
    enums::Mode,
    image::{JpegImage, SvgImage},
};
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    // Init backend
    let (mut painter, egui_state) = egui_backend::with_fltk(&mut win);
    let state = Rc::from(RefCell::from(egui_state));

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

    let retained_egui_image = JpegImage::load("screenshots/egui.jpg")
        .unwrap()
        .egui_image("egui.jpg", egui::TextureOptions::LINEAR)
        .unwrap();
    let retained_egui_image_svg = SvgImage::load("screenshots/fingerprint.svg")
        .unwrap()
        .egui_svg_image("fingerprint.svg", egui::TextureOptions::LINEAR)
        .unwrap();

    let egui_ctx = egui::Context::default();
    let start_time = Instant::now();
    let mut quit = false;

    while fltk_app.wait() {
        // Clear the screen to dark red
        let gl = painter.gl().as_ref();
        draw_background(gl);

        let mut state = state.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let egui_output = egui_ctx.run(state.take_input(), |ctx| {
            egui::CentralPanel::default().show(&ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(Label::new("this is fingerprint.svg"));
                    retained_egui_image_svg.show(ui);
                    ui.add(Label::new("this is egui.jpg"));
                    retained_egui_image.show(ui);
                    if ui
                        .button("Quit?")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        quit = true;
                    }
                });
            });
        });

        if egui_output.repaint_after.is_zero() || state.window_resized() {
            state.fuse_output(&mut win, egui_output.platform_output);
            let meshes = egui_ctx.tessellate(egui_output.shapes);

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

        if quit {
            break;
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
