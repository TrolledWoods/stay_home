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

use prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	t * (b - a) + a
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
	MoveLeft,
	MoveRight,
	MoveUp,
	MoveDown,
	Confirm,
	Restart,
}

fn main() {
	let level = r#"
	@$..HH
	...#.$
	...#..
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
	keybindings.insert(19, Input::Restart); // 'R'

	let mut graphics = graphics::Graphics::new(&display);
	let original_level = level::Level::from_string(level).unwrap();
	let mut level = original_level.clone();
	let mut level_graphics = 
		level_graphics::LevelGraphics::new(&graphics, &level);

	let mut cached_input = None;

	let mut previous_frame = Instant::now();

	let mut events = VecDeque::new();
	events_loop.run(move |event, _, control_flow| {
		let current_frame = Instant::now();
		let mut dt = (current_frame - previous_frame).as_micros() as f32 
			/ 1_000_000.0;
		previous_frame = current_frame;

		*control_flow = glutin::event_loop::ControlFlow::WaitUntil(current_frame + std::time::Duration::from_millis(1));

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
						if keybind == Input::Restart {
							level = original_level.clone();
							level_graphics
								.reset(&mut graphics, &level);
						}

						cached_input = Some(keybind);
					}
				}else {
					println!("Unknown key scancode: '{}'", scancode);
				}
			}
			_ => (),
		}

		if level_graphics.animations.len() == 0 {
			if let Some(input) = cached_input.take() {
				level.input(input, &mut events);
				for event in events.drain(..) {
					level_graphics.animations.push_back((0.0, event));
				}
			}
		}

		let mut frame = display.draw();
		frame.clear_color(0.3, 0.3, 0.5, 1.0);
		level_graphics.render_level(&graphics, &mut frame, aspect, &mut level, dt);
		frame.finish().unwrap();
	});
}
