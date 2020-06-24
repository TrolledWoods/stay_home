mod level;
mod graphics;
mod textures;
mod level_graphics;
mod matrix;

mod prelude {
	pub use glium::*;
	pub use glutin::event::{KeyboardInput, ElementState};
	pub use crate::level::{Level, Event};
	pub use crate::Input;
	pub use crate::lerp;
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	t * (b - a) + a
}

use prelude::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
	MoveLeft,
	MoveRight,
	MoveUp,
	MoveDown,
	Confirm,
}

fn main() {
	let level = r#"
	....@.....
	......##..
	H.........
	......#...
	........$.
	##.##.....
	"#;
	
	let mut aspect = 1024.0 / 768.0;
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Making quarantine great again!!!!");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

	let mut keybindings = HashMap::new();
	keybindings.insert(72, Input::MoveUp);
	keybindings.insert(75, Input::MoveLeft);
	keybindings.insert(80, Input::MoveDown);
	keybindings.insert(77, Input::MoveRight);
	keybindings.insert(28, Input::Confirm);
	keybindings.insert(57, Input::Confirm);

	let mut graphics = graphics::Graphics::new(&display);
	let mut level = level::Level::from_string(level).unwrap();
	let mut level_graphics = 
		level_graphics::LevelGraphics::new(&graphics, &level);

	let mut events = VecDeque::new();
	events_loop.run(move |event, _, control_flow| {
		use glutin::event::{Event, WindowEvent};
		match event {
			Event::WindowEvent {
				event: WindowEvent::Resized(size),
				..
			} => {
				aspect = size.width as f32 / size.height as f32;
			}
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput {
					input: KeyboardInput { state, scancode, .. },
					..
				},
				..
			} => {
				if let Some(&keybind) = keybindings.get(&scancode) {
					if state == ElementState::Pressed {
						level.input(keybind, &mut events);
						for event in events.drain(..) {
							level_graphics.animations.push_back((0.0, event));
						}
					}
				}else {
					println!("Unknown key scancode: '{}'", scancode);
				}
			}
			_ => (),
		}

		let mut frame = display.draw();
		frame.clear_color(0.3, 0.3, 0.5, 1.0);
		level_graphics.render_level(&graphics, &mut frame, aspect, &mut level, 0.01);
		frame.finish().unwrap();
	});
}
