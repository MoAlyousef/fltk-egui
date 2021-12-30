use egui_backend::{
    egui,
    fltk::{enums::*, prelude::*, *},
    gl, DpiScaling, EguiImageConvertible,
};
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let a = app::App::default();
    let mut win = window::GlWindow::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    win.set_mode(Mode::Opengl3);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let (painter, egui_input_state) = egui_backend::with_fltk(&mut win, DpiScaling::Custom(1.5));
    let mut egui_ctx = egui::CtxRef::default();

    let state = Rc::from(RefCell::from(egui_input_state));
    let painter = Rc::from(RefCell::from(painter));

    win.handle({
        let state = state.clone();
        let painter = painter.clone();
        move |win, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag => {
                let mut state = state.borrow_mut();
                state.fuse_input(win, ev, &mut painter.borrow_mut());
                true
            }
            _ => false,
        }
    });

    let image = image::JpegImage::load("screenshots/egui.jpg").unwrap();
    let (image, _) = image
        .to_egui_image(&mut painter.borrow_mut(), (300, 300), false)
        .unwrap();

    let start_time = Instant::now();
    let mut quit = false;

    while a.wait() {
        let mut state = state.borrow_mut();
        let mut painter = painter.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        let (egui_output, shapes) = egui_ctx.run(state.input.take(), |ctx| {
            unsafe {
                // Clear the screen to black
                gl::ClearColor(0.6, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.heading("My egui Application");
                ui.add(image);
                if ui
                    .button("Quit?")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    quit = true;
                }
            });
        });

        state.fuse_output(&mut win, &egui_output);

        let meshes = egui_ctx.tessellate(shapes);

        //Draw egui texture
        painter.paint_jobs(None, meshes, &egui_ctx.font_image());

        win.swap_buffers();
        win.flush();

        if egui_output.needs_repaint {
            app::awake()
        } else if quit {
            break;
        }
    }
}
