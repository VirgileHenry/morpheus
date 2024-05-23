use morpheus::world::components::transform::Transform;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub use morpheus::*;

fn main() {

    let event_loop = EventLoop::new().unwrap();
    
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let start_size = (window.inner_size().width, window.inner_size().height);

    let mut renderer = match renderer::Renderer::new(&window, start_size) {
        Ok(renderer) => renderer,
        Err(e) => {
            println!("Unable to create renderer: {e:?}");
            std::process::exit(1);
        }
    };

    renderer.load_csg(
        0,
        csg::csg!(
            csg::BinOp::Inter => {
                csg::Primitive::sphere(0.3)
            } {
                csg::Primitive::sphere(0.3).at(glam::Vec3::new(0.0, 0.2, 0.0))
            }
        )
    );

    renderer.create_obj(Transform::origin().rotated(glam::Quat::from_axis_angle(glam::Vec3::Y, 0.3)), 0);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => renderer.resize((physical_size.width, physical_size.height)),
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => match renderer.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => todo!("handle surface lost"),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            },
            // key handling
            Event::WindowEvent {
                event: winit::event::WindowEvent::KeyboardInput {
                    event: winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        state: winit::event::ElementState::Pressed, ..
                    }, ..
                }, ..
            } => elwt.exit(),
            Event::WindowEvent {
                event: winit::event::WindowEvent::KeyboardInput {
                    event: winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF),
                        state: winit::event::ElementState::Pressed, ..
                    }, ..
                }, ..
            } => window.set_fullscreen(match window.fullscreen() {
                Some(_) => None,
                None => Some(winit::window::Fullscreen::Borderless(None)),
            }),
            _ => ()
        }
    });
}