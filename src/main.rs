mod game_state;
mod level;
mod graphics;
mod textures;
mod level_graphics;
mod matrix;

mod prelude {
	pub use glium::*;
	pub use glutin::event::{KeyboardInput, ElementState};
	pub use crate::level::Level;
	pub use crate::level_graphics::LevelGraphics;
	pub use crate::graphics::Graphics;
	pub use crate::{Input, lerp};
}

use prelude::*;
use std::collections::HashMap;
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
	// let level = r#"
	// @$..HH
	// ...#.$
	// ...#..
	// "#;
	let mut aspect = 1024.0 / 768.0;
    let events_loop = glium::glutin::event_loop::EventLoop::new();
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
	keybindings.insert(56, Input::Confirm);
	keybindings.insert(19, Input::Restart); // 'R'

	let mut graphics = graphics::Graphics::new(&display);

	let mut state = game_state::GameState::PlayingLevel(
		game_state::LevelPlayer::new(
			vec![
				"levels/experiment.txt".parse().unwrap(),
				"levels/start.txt".parse().unwrap(),
				"levels/walk_through_empty_homes.txt".parse().unwrap(),
				"levels/filled_houses.txt".parse().unwrap(),
				"levels/indirect_access.txt".parse().unwrap(),
				"levels/priorities.txt".parse().unwrap(),
				"levels/cake_trouble.txt".parse().unwrap(),
				"levels/crazy_cake_level.txt".parse().unwrap(),
			], 
			&mut graphics,
		).unwrap()
	);

	let mut previous_frame = Instant::now();
	events_loop.run(move |event, _, control_flow| {
		let current_frame = Instant::now();
		let dt = (current_frame - previous_frame).as_micros() as f32 
			/ 1_000_000.0;
		previous_frame = current_frame;

		*control_flow = glutin::event_loop::ControlFlow::WaitUntil(current_frame + std::time::Duration::from_millis(5));

		use glutin::event::{Event, WindowEvent};
		match event {
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				*control_flow = glutin::event_loop::ControlFlow::Exit;
			}
			Event::WindowEvent {
				event: WindowEvent::Resized(size),
				..
			} => {
				aspect = size.width as f32 / size.height as f32;
			}
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput {
					input: KeyboardInput { state: key_state, scancode, .. },
					..
				},
				..
			} => {
				if let Some(&keybind) = keybindings.get(&scancode) {
					if key_state == ElementState::Pressed {
						state.input(&mut graphics, keybind).unwrap();
					}
				}else {
					println!("Unknown key scancode: '{}'", scancode);
				}
			}
			_ => (),
		}

		let mut frame = display.draw();
		state.render(&mut frame, &mut graphics, aspect, dt).unwrap();
		frame.finish().unwrap();
	});
}
