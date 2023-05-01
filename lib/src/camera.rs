pub struct Camera {
    pos_x: f32,
    pos_y: f32,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos_x: -0.5,
            pos_y: 0.0,
            zoom: 1.0,
        }
    }

    /// Inverse view matrix, transforms from canvas space, to the space of the coordinate system.
    ///
    /// Translates and zooms. Columnwise defined.
    pub fn inv_view(&self) -> [[f32; 2]; 3] {
        // [ 1/z  0   tx]    | x |   | x/z + tx |
        // [  0  1/z  ty]  x | y | = | y/z - ty |
        //                   | 1 |
        [
            [1. / self.zoom, 0.],
            [0., 1. / self.zoom],
            [self.pos_x, self.pos_y],
        ]
    }

    pub fn zoom(&mut self, factor: f32) {
        self.zoom *= factor;
    }

    pub fn change_pos(&mut self, delta_x: f32, delta_y: f32) {
        self.pos_x += delta_x / self.zoom;
        self.pos_y += delta_y / self.zoom
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
