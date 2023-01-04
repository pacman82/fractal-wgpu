pub struct Camera {
    pos_x : f32,
    pos_y : f32,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Self{
        Camera {
            pos_x : -0.5,
            pos_y : 0.0,
            zoom: 1.0,
        }
    }

    /// Inverse view matrix, transforms from canvas space, to the space of the coordinate system.
    pub fn inv_view(&self) -> [[f32;3];3] {
        [
            [1. / self.zoom, 0., 0.],
            [0., 1. / self.zoom, 0.],
            [self.pos_x, self.pos_y, 1.],
        ]
    }
}