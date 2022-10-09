use cgmath::Zero;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Clone)]
pub struct CameraController {
    view_ortho_matrix: cgmath::Matrix4<f32>,
    center_x: f32, center_y: f32,
    scale: f32,
    aspect: (f32, f32)
}

impl CameraController {
    pub fn new(center_x: f32, center_y: f32, aspect: (f32, f32)) -> Self {
        //generate a default scale from the aspect, assuming each unit is 10px

        let mut instance = Self {
            view_ortho_matrix: cgmath::Matrix4::zero(),
            center_x,
            center_y,
            aspect,
            scale: 10.0,
        };
        instance.update();

        instance
    }

    pub fn get_view_ortho_matrix(&self) -> cgmath::Matrix4<f32> {
        self.view_ortho_matrix
    }

    pub fn _translate(&mut self, dx: f32, dy: f32) {
        self.center_x += dx;
        self.center_y += dy;
        self.update();
    }

    pub fn resize(&mut self, aspect: (f32, f32)) {
        self.aspect = aspect;
        self.update();
    }

    fn update(&mut self) {
        //update with the aspect ratio here
        let x_steps = self.aspect.0 / self.scale;
        let y_steps = self.aspect.0 / self.scale;
        let left = -(x_steps / 2.0);
        let right = x_steps / 2.0;
        let bottom = -(y_steps / 2.0);
        let top = y_steps / 2.0;

        self.view_ortho_matrix = Self::build_view(self.center_x, self.center_y, left, right, bottom, top);
    }

    fn build_view(
        center_x: f32,
        center_y: f32,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32
    ) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_lh(
            (center_x, center_y, 0f32).into(),
            (center_x, center_y, 1f32).into(),
            cgmath::Vector3::<f32>::unit_y(),
        );

        let ortho = cgmath::ortho(left, right, bottom, top, 1f32, -1f32);

        OPENGL_TO_WGPU_MATRIX * ortho * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_ortho: [[f32; 4]; 4],
}

impl From<CameraController> for CameraUniform {
    fn from(value: CameraController) -> Self {
        Self {
            view_ortho: value.get_view_ortho_matrix().into()
        }
    }
}
