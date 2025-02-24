use std::time::{Duration, Instant};

use winit::{event::{ElementState, KeyEvent}, keyboard::{KeyCode, PhysicalKey}};

use fractal_wgpu_lib::Camera;

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
    inc_iter: bool,
    dec_iter: bool,
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
            inc_iter: false,
            dec_iter: false,
        }
    }

    pub fn track_button_presses(&mut self, input: KeyEvent) {
        let KeyEvent {
            state,
            physical_key, 
            ..
        } = input;
        if let PhysicalKey::Code(keycode) = physical_key {
            let is_pressed = state == ElementState::Pressed;
            match keycode {
                KeyCode::ArrowLeft => self.left = is_pressed,
                KeyCode::ArrowUp => self.up = is_pressed,
                KeyCode::ArrowRight => self.right = is_pressed,
                KeyCode::ArrowDown => self.down = is_pressed,
                KeyCode::Period => self.zoom_in = is_pressed,
                KeyCode::Comma => self.zoom_out = is_pressed,
                KeyCode::KeyM => self.inc_iter = is_pressed,
                KeyCode::KeyN => self.dec_iter = is_pressed,
                _ => (),
            }
            if self.outdated_since.is_none() && self.picture_changes() {
                self.outdated_since = Some(Instant::now())
            }
        };
    }

    pub fn update_scene(&mut self, camera: &mut Camera, iterations: &mut f32) {
        let now = Instant::now();
        if let Some(outdated_since) = self.outdated_since {
            let delta_time = now - outdated_since;
            self.update_camera(delta_time, camera);
            // Iterations
            //
            // Change iterations in log space since we perceive the difference between 1 and 100
            // iterations way stronger than the difference between 101 and 200.
            let delta_iter = 0.5 * delta_time.as_secs_f32();
            let mut ln_iter = iterations.ln();
            if self.inc_iter {
                ln_iter += delta_iter;
                ln_iter = ln_iter.min(10.0);
            }
            if self.dec_iter {
                ln_iter -= delta_iter;
                ln_iter = ln_iter.max(0.0);
            }
            *iterations = ln_iter.exp()
        }
        if self.picture_changes() {
            self.outdated_since = Some(now);
        } else {
            self.outdated_since = None;
        }
    }

    fn update_camera(&mut self, delta_time: Duration, camera: &mut Camera) {
        let delta_pos = 1.0 * delta_time.as_secs_f32();
        let delta_zoom = 1.0 + 0.4 * delta_time.as_secs_f32();
        // Camera
        let mut delta_x = 0.;
        let mut delta_y = 0.;
        let mut zoom = 1.0;
        if self.left {
            delta_x -= delta_pos;
        }
        if self.right {
            delta_x += delta_pos;
        }
        if self.up {
            delta_y += delta_pos;
        }
        if self.down {
            delta_y -= delta_pos;
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

    pub fn picture_changes(&self) -> bool {
        self.up
            || self.down
            || self.left
            || self.right
            || self.zoom_in
            || self.zoom_out
            || self.inc_iter
            || self.dec_iter
    }
}
