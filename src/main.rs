#![allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;

use drawer::Drawer;
use minifb::{Key, Window, WindowOptions, ScaleMode, Scale};

mod drawer;
pub mod math;

const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 360;

fn main() {
    let window = Rc::new(RefCell::new(setup_window(SCREEN_WIDTH, SCREEN_HEIGHT)));

    window.borrow_mut().set_position(-10, 0);

    let mut drawer = Drawer::new(SCREEN_WIDTH, SCREEN_HEIGHT, window.clone());

    let mut is_open = window.borrow().is_open();
    let mut is_down = window.borrow().is_key_down(Key::Escape);

    let (near, far, fov_deg, aspect_ratio) = drawer.ready();

    let mut last_instant = Instant::now();

    while is_open && !is_down {
        let now = Instant::now();
        let delta = now.duration_since(last_instant).as_secs_f32();
        last_instant = now;
        let fps = 1.0 / delta;

        drawer.update(delta);

        drawer.draw_string(10, 10, format!("NEAR: {}", near).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 25, format!("FAR: {}", far).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 40, format!("FOV: {}", fov_deg).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 55, format!("ASPECT RATIO: {}", aspect_ratio).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 70, format!("DELTA: {}", delta).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 85, format!("FPS: {:.2}", fps).as_str(), 0xFFFFFF);

        drawer.draw_string(10, 115, format!("YAW: {}", drawer.yaw).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 130, format!("PITCH: {}", drawer.pitch).as_str(), 0xFFFFFF);
        drawer.draw_string(10, 145, format!("LOOK DIR: {}", drawer.look_dir).as_str(), 0xFFFFFF);

        window.borrow_mut().update_with_buffer(drawer.buffer.as_slice(), drawer.width, drawer.height).unwrap();

        is_open = window.borrow().is_open();
        is_down = window.borrow().is_key_down(Key::Escape);
    }
}

fn setup_window(width: usize, height: usize) -> Window {
    Window::new(
        "Tests",
        width,
        height,
        WindowOptions {
            resize: true,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e)
    })
}