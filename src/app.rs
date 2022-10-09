use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{graph::GraphRenderer, renderer::Renderer};

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
        let mut renderer = GraphRenderer::new(&window).await;

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
                WindowEvent::Resized(physical_size) => {
                    //resize the renderer
                    //resize the ECS
                    renderer.resize(Some(*physical_size));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(Some(**new_inner_size));
                },
                e => {
                    //pass the event to the renderer and any components
                    renderer.event(e);
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
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
