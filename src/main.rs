mod level;
mod graphics;
mod textures;

mod prelude {
	pub use glium::*;
	pub use crate::level::Level;
}

use prelude::*;

fn main() {
	let level = r#"
	..........
	......##..
	H.........
	......#...
	..........
	##.##.....
	"#;
	
	let mut aspect = 1024.0 / 768.0;
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Making quarantine great again!!!!");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

	let textures = textures::Textures::load("assets/", &display).unwrap();
	
	let mut level = level::Level::from_string(level).unwrap();
	let mut graphics = graphics::Graphics::new(&display);

	events_loop.run(move |event, _, control_flow| {
		use glutin::event::{Event, WindowEvent};
		match event {
			Event::WindowEvent {
				event: WindowEvent::Resized(size),
				..
			} => {
				aspect = size.width as f32 / size.height as f32;
			}
			_ => (),
		}

		let mut frame = display.draw();
		frame.clear_color(0.3, 0.3, 0.5, 1.0);
		graphics.render_level(&display, &mut frame, &textures, aspect, &mut level);
		frame.finish().unwrap();
	});
}
