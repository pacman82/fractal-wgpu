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
    /// 
    /// Translates and zooms. Columnwise defined.
    pub fn inv_view(&self) -> [[f32;2];3] {
        // [ 1/z  0   tx]    | x |   | x/z + tx |
        // [  0  1/z  ty]  x | y | = | y/z - ty |
        [
            [1. / self.zoom, 0.],
            [0., 1. / self.zoom],
            [self.pos_x, self.pos_y],
        ]
    }
}