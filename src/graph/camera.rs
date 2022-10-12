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
    resolution: PhysicalSize<u32>,
    aspect: (f32, f32),
    //these bounds are in graph space but relative to the camera
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,

    scale: f32,
    //for handling events
    pressed: bool, //for if the left mouse button is pressed
    start_press: PhysicalPosition<f64>,
    start_position: (f32, f32),
    cursor_pos: PhysicalPosition<f64>,
}

impl CameraController {
    pub fn new(center_x: f32, center_y: f32, resolution: PhysicalSize<u32>) -> Self {
        //generate a default scale from the aspect, assuming each unit is 10px
        let mut instance = Self {
            center_x,
            center_y,
            resolution,
            aspect: (0f32, 0f32),
            //view elements
            left: 0f32,
            right: 0f32,
            bottom: 0f32,
            top: 0f32,

            scale: 2.0,

            pressed: false,
            start_press: (-1f32, -1f32).into(), //start press in screen space
            start_position: (0f32, 0f32),       //the center when we started pressing
            cursor_pos: (-1f32, -1f32).into(),  //cursor position passed around in events
        };
        instance.update();
        //instance.zoom((0f32, 0f32).into(), 0f32); //update the zoom
        instance
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.resolution = new_size;
        self.update();
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
                self.start_press = self.cursor_pos; //set start press in here
                self.start_position = (self.center_x, self.center_y);
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
            CursorMoved { position, .. } => {
                self.cursor_pos = *position;
                if self.pressed {
                    //convert position to graph space
                    self.center_x = self.start_position.0
                        - (position.x - self.start_press.x) as f32 / self.scale;
                    self.center_y = self.start_position.1
                        + (position.y - self.start_press.y) as f32 / self.scale;
                    self.update();
                    true
                } else {
                    false
                }
            }
            MouseWheel {
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }),
                ..
            } => {
                //use the y-scroll for a zoom coefficient
                let dzoom = (-*y / 1000.0f64) as f32;
                self.zoom(self.cursor_pos, dzoom);
                true
            }
            //handle the mouse cursor movements if pressd
            _ => false,
        }
    }

    fn zoom(&mut self, position: PhysicalPosition<f64>, dzoom: f32) {
        //zoom in on a point, dont really get this code
        //calculate a global top
        let zoom = 2.0f32.powf(-dzoom);

        //convert the position to graph space for later
        let x = self.left + position.x as f32 / self.scale;
        let y = self.top - position.y as f32 / self.scale;

        //calculate the old positio
        let og_x = position.x as f32;
        let og_y = position.y as f32;

        self.scale *= zoom;
        self.update();

        let new_x = (x - self.left) * self.scale;
        let new_y = (self.top - y) * self.scale;

        //and then translate so that the mouse stays at the same position
        self.center_x += (new_x - og_x) / self.scale;
        self.center_y -= (new_y - og_y) / self.scale;
        self.update();
    }

    fn update(&mut self) {
        //update the bounds
        let right_relative = self.resolution.width as f32 / 2.0 / self.scale;
        let top_relative = self.resolution.height as f32 / 2.0 / self.scale;

        self.left = self.center_x - right_relative;
        self.right = self.center_x + right_relative;
        self.bottom = self.center_y - top_relative;
        self.top = self.center_y + top_relative;

        //update the aspect
        self.aspect.0 = (self.right - self.left) / self.resolution.width as f32;
        self.aspect.1 = (self.top - self.bottom) / self.resolution.height as f32;
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
        //relative top, bottom, left, right
        let left = self.left - self.center_x;
        let right = self.right - self.center_x;
        let bottom = self.bottom - self.center_y;
        let top = self.top - self.center_y;

        let ortho = cgmath::ortho(left, right, bottom, top, 1f32, -1f32);
        let combined_matrix = OPENGL_TO_WGPU_MATRIX * ortho * view;

        CameraMatrix {
            view_ortho: combined_matrix.into(),
        }
    }
}

impl Into<View> for CameraController {
    fn into(self) -> View {
        View {
            left: self.left,
            right: self.right,
            bottom: self.bottom,
            top: self.top,
            center_x: self.center_x,
            center_y: self.center_y,
            aspect: self.aspect,
        }
    }
}
