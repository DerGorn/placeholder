use winit::dpi::PhysicalSize;

pub fn static_camera(view_size: PhysicalSize<f32>) -> [[f32; 2]; 3] {
    [
        [2.0 / view_size.width, 0.0],
        [0.0, 2.0 / view_size.height],
        [0.0, 0.0],
    ]
}
