use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, MouseButton, MouseScrollDelta,
        WindowEvent::{self, CursorMoved, MouseInput, MouseWheel},
    },
};

use super::view::View;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

//TODO: Zooming in always zooms in on the center, make it move the camera towards where your zooming and
#[derive(Clone)]
pub struct CameraController {
    center_x: f32,
    center_y: f32,
    aspect: PhysicalSize<u32>,
    //these bounds are in graph space but relative to the camera
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    scale: f32,
    //for handling events
    pressed: bool, //for if the left mouse button is pressed
    start_press: PhysicalPosition<f64>,
}

impl CameraController {
    pub fn new(center_x: f32, center_y: f32, aspect: PhysicalSize<u32>) -> Self {
        //generate a default scale from the aspect, assuming each unit is 10px
        let mut instance = Self {
            center_x,
            center_y,
            aspect,
            //view elements 
            left: 0f32,
            right: 0f32,
            bottom: 0f32,
            top: 0f32,
            scale: 10.0,
            pressed: false,
            start_press: (-1f64, -1f64).into(),
        };
        instance.update_bounds();
        instance
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.aspect = new_size;
        self.update_bounds();
    }

    //return true if the view changed
    pub fn event(&mut self, event: &WindowEvent) -> bool {
        match event {
            //toggle pressed
            MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.pressed = true;
                self.start_press = PhysicalPosition { x: -1f64, y: -1f64 }; //set start press in here
                false
            }
            MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.pressed = false;
                false
            }
            CursorMoved { position, .. } if self.pressed => {
                //set our start press if start_press is 0
                if self.start_press.x != -1f64 {
                    let dx = -(position.x - self.start_press.x) as f32 / self.scale as f32;
                    let dy = (position.y - self.start_press.y) as f32 / self.scale as f32;
                    self.translate(dx, dy);
                    self.start_press = *position;
                    true
                }  else {
                    self.start_press = *position;
                    false
                }
            }
            MouseWheel {
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }),
                ..
            } => {
                //use the y-scroll for a zoom coefficient
                let zoom_coefficient = (-*y / 100.0f64) as f32;
                //get the mouse position here, convert to graph space and pass to zoom
                self.zoom((0f32, 0f32), zoom_coefficient);
                true
            }
            //handle the mouse cursor movements if pressd
            _ => false,
        }
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        self.center_x += dx;
        self.center_y += dy;
    }

    fn zoom(&mut self, _position: (f32, f32), zoom_coefficient: f32) {
        //for now jankily change the scale based on the zoom coefficent and resize with the new scale
        //zoom in exponential space
        println!("zoom coefficient: {}", zoom_coefficient);
        self.scale += zoom_coefficient;
        if self.scale < 1.1f32 {
            self.scale = 1.1f32;
        }
        self.update_bounds();
    }

    fn update_bounds(&mut self) {
        //resize keeping the center in the center
        let x_steps = self.aspect.width as f32 / self.scale;
        let y_steps = self.aspect.height as f32 / self.scale;

        self.left = -(x_steps / 2.0);
        self.right = x_steps / 2.0;
        self.bottom = -(y_steps / 2.0);
        self.top = y_steps / 2.0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraMatrix {
    pub view_ortho: [[f32; 4]; 4],
}

//cam controller should be cloned before calling into as 
//generally we want to keep these things alive, this should be a relatively cheap clone

//these should probably be into traits not froms
impl Into<CameraMatrix> for CameraController {
    fn into(self) -> CameraMatrix {
        //construct the matrices here
        let view = cgmath::Matrix4::look_at_lh(
            (self.center_x, self.center_y, 0f32).into(),
            (self.center_x, self.center_y, 1f32).into(),
            cgmath::Vector3::<f32>::unit_y(),
        );
        let ortho = cgmath::ortho(self.left, self.right, self.bottom, self.top, 1f32, -1f32);
        let combined_matrix = OPENGL_TO_WGPU_MATRIX * ortho * view;

        CameraMatrix {
            view_ortho: combined_matrix.into(),
        }
    }
}

impl Into<View> for CameraController {
    fn into(self) -> View {
        //put local camera bounds into a graph space
        View {
            left: self.center_x + self.left,
            right: self.center_x + self.right,
            bottom: self.center_y + self.bottom,
            top: self.center_y + self.top,
            scale: self.scale,
        }
    }
}
