#![feature(decl_macro)]

use std::{
    sync::{atomic::Ordering, Arc},
    thread,
};

use anyhow::Result;
use winit::{
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod app;
mod args;
mod cues;
mod time;
use app::App;

fn main() -> Result<()> {
    // Create window
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("video-presenter")
        .build(&event_loop)
        .unwrap();
    // Get window handle.
    // Its used for telling mpv where to render.
    let wid = u64::from(window.id());

    // Create the app instance, this inits mpv
    let app = Arc::new(App::new(wid)?);
    window.set_title(&format!("video-presenter \u{2013} {}", app.video_name()));

    // Start the mpv event loop
    let app2 = app.clone();
    thread::spawn(move || app2.event_loop());

    // Start the winit event loop
    event_loop.run(move |event, _window, control_flow| {
        if input.update(&event) {
            if input.close_requested() || input.destroyed() {
                *control_flow = ControlFlow::Exit;
            }

            if input.key_pressed(VirtualKeyCode::P) {
                let paused = app.mpv.get_property::<bool>("pause").unwrap();
                app.mpv.set_property("pause", !paused).unwrap();
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                let paused = app.mpv.get_property::<bool>("pause").unwrap();

                if paused {
                    app.mpv.unpause().unwrap();
                } else {
                    app.seek_f().unwrap();
                }
            }

            if input.key_pressed(VirtualKeyCode::Right) {
                app.mpv.pause().unwrap();
                app.seek_f().unwrap();

                let cue = app.current_cue.load(Ordering::Relaxed);
                app.info(format!("#{cue}"));
            }

            if input.key_pressed(VirtualKeyCode::Left) {
                app.mpv.pause().unwrap();
                app.seek_r().unwrap();

                let cue = app.current_cue.load(Ordering::Relaxed);
                app.info(format!("#{cue}"));
            }

            if input.key_pressed(VirtualKeyCode::Period) {
                app.mpv.seek_frame().unwrap();
                app.auto_cue();
            }

            if input.key_pressed(VirtualKeyCode::Comma) {
                app.mpv.seek_frame_backward().unwrap();
                app.auto_cue();
            }
        }
    });
}
