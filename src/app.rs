use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{graph::{GraphCameraController, GraphRenderer}, renderer::Renderer};

pub struct App;

impl App {
    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub async fn run() {

        env_logger::init();
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Grapher")
            .build(&event_loop)
            .unwrap();

        //create our renderer and our graph here
        let mut cam_controller = GraphCameraController::new(0f32, 0f32, window.inner_size());
        let mut renderer = GraphRenderer::new(&window, &cam_controller).await;

        //store a flag for if our view changed and then update all the components before rendering
        let mut view_changed = false;
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                //handle resizes
                WindowEvent::Resized(new_size) => {
                    //resize the renderer
                    //resize the ECS
                    renderer.resize(Some(*new_size));
                    cam_controller.resize(*new_size);
                    view_changed = true;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(Some(**new_inner_size));
                    cam_controller.resize(**new_inner_size);
                    view_changed = true;
                },
                e => {
                    //pass the events to our cam controller (which is kind of a component)
                    view_changed = cam_controller.event(e);
                    //pass events to our other components in theory
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if view_changed {
                    //update our renderer and other components here
                    renderer.update_view(&cam_controller);
                    //update our other components here
                    
                    view_changed = false;
                }
                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        });
    }
}
