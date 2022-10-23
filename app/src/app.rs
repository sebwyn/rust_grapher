use bevy_ecs::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::graph::GraphRenderContext;

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

        //create our world here, and add renderables to it
        let world = World::new();
        
        //create our renderer and our graph here
        let mut graph_render_context = GraphRenderContext::new(&window).await;

        //store a flag for if our view changed and then update all the components before rendering
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
                    renderer.resize(Some(*new_size));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(Some(**new_inner_size));
                },
                e => {
                    renderer.event(e);
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                renderer.update();
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
