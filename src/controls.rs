use std::time::Instant;

use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::camera::Camera;

/// Keep track of which buttons are pressed and decide how much the camera should move from one
/// frame to the next.
pub struct Controls {
    // Since then is the picture currently displayed in the canvas outdated? We use this variable to
    // check how much we adapt the camera positions between frames. If the picture is currently
    // unchanging we set this to `None`.
    outdated_since: Option<Instant>,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    zoom_in: bool,
    zoom_out: bool,
}

impl Controls {
    pub fn new() -> Self {
        Controls {
            outdated_since: None,
            up: false,
            down: false,
            left: false,
            right: false,
            zoom_in: false,
            zoom_out: false,
        }
    }

    pub fn track_button_presses(&mut self, input: KeyboardInput) {
        let KeyboardInput {
            scancode: _,
            state,
            virtual_keycode,
            ..
        } = input;
        if let Some(keycode) = virtual_keycode {
            let is_pressed = state == ElementState::Pressed;
            match keycode {
                VirtualKeyCode::Left => self.left = is_pressed,
                VirtualKeyCode::Up => self.up = is_pressed,
                VirtualKeyCode::Right => self.right = is_pressed,
                VirtualKeyCode::Down => self.down = is_pressed,
                VirtualKeyCode::Period => self.zoom_in = is_pressed,
                VirtualKeyCode::Comma => self.zoom_out = is_pressed,
                _ => (),
            }
            if self.outdated_since.is_none() && self.picture_changes() {
                self.outdated_since = Some(Instant::now())
            }
        };
    }

    pub fn change_camera(&mut self, camera: &mut Camera) {
        let now = Instant::now();
        if let Some(outdated_since) = self.outdated_since {
            let delta_time = now - outdated_since;
            let delta = 1.0 * delta_time.as_secs_f32();
            let delta_zoom = 1.0 + 0.4 * delta_time.as_secs_f32();
            let mut delta_x = 0.;
            let mut delta_y = 0.;
            let mut zoom = 1.0;
            if self.left {
                delta_x -= delta;
            }
            if self.right {
                delta_x += delta;
            }
            if self.up {
                delta_y += delta;
            }
            if self.down {
                delta_y -= delta;
            }
            if self.zoom_in {
                zoom *= delta_zoom;
            }
            if self.zoom_out {
                zoom /= delta_zoom;
            }
            camera.change_pos(delta_x, delta_y);
            camera.zoom(zoom);
        }
        if self.picture_changes() {
            self.outdated_since = Some(now);
        } else {
            self.outdated_since = None;
        }
    }

    pub fn picture_changes(&self) -> bool {
        self.up || self.down || self.left || self.right || self.zoom_in || self.zoom_out
    }
}
