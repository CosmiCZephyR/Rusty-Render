#![allow(dead_code)]
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use drawer::Drawer;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

mod drawer;
pub mod math;
mod camera;

const SCREEN_WIDTH: usize = 1920;
const SCREEN_HEIGHT: usize = 1080;

fn main() {
    let window = Rc::new(RefCell::new(setup_window(SCREEN_WIDTH, SCREEN_HEIGHT)));

    window.borrow_mut().set_position(-10, 0);

    let mut drawer = Drawer::new(SCREEN_WIDTH, SCREEN_HEIGHT, window.clone());

    let mut is_open = window.borrow().is_open();
    let mut is_down = window.borrow().is_key_down(Key::Escape);

    let (near, far, fov_deg, aspect_ratio) = drawer.ready();

    let mut last_instant = Instant::now();

    window
        .borrow_mut()
        .limit_update_rate(
            Option::from(
                Duration::from_micros(16666)
            )
        );

    while is_open && !is_down {
        let now = Instant::now();
        let delta = now.duration_since(last_instant).as_secs_f32();
        last_instant = now;
        let fps = 1.0 / delta;

        drawer.update(delta);

        draw_debug(&mut drawer, near, far, fov_deg, aspect_ratio, delta, fps);

        window
            .borrow_mut()
            .update_with_buffer(drawer.buffer.as_slice(), drawer.width, drawer.height)
            .unwrap();

        is_open = window.borrow().is_open();
        is_down = window.borrow().is_key_down(Key::Escape);
    }
}

fn draw_debug(
    drawer: &mut Drawer,
    near: f32,
    far: f32,
    fov_deg: f32,
    aspect_ratio: f32,
    delta: f32,
    fps: f32,
) {
    drawer.draw_string(10, 10, format!("NEAR: {}", near).as_str(), 0xFFFFFF);
    drawer.draw_string(10, 25, format!("FAR: {}", far).as_str(), 0xFFFFFF);
    drawer.draw_string(10, 40, format!("FOV: {}", fov_deg).as_str(), 0xFFFFFF);
    drawer.draw_string(
        10,
        55,
        format!("ASPECT RATIO: {}", aspect_ratio).as_str(),
        0xFFFFFF,
    );
    drawer.draw_string(10, 70, format!("DELTA: {}", delta).as_str(), 0xFFFFFF);
    drawer.draw_string(10, 85, format!("FPS: {:.2}", fps).as_str(), 0xFFFFFF);
    drawer.draw_string(10, 115, format!("YAW: {}", drawer.camera.yaw).as_str(), 0xFFFFFF);
    drawer.draw_string(
        10,
        130,
        format!("PITCH: {}", drawer.camera.pitch).as_str(),
        0xFFFFFF,
    );
    drawer.draw_string(
        10,
        145,
        format!("LOOK DIR: {}", drawer.camera.look_dir).as_str(),
        0xFFFFFF,
    );
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
    .unwrap_or_else(|e| panic!("{}", e))
}
