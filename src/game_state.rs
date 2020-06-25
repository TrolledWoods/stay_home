use crate::prelude::*;
use std::path::PathBuf;
use std::collections::VecDeque;
use std::fs;

pub enum GameState {
	// MainMenu(MainMenu),
	PlayingLevel(LevelPlayer),
	// PauseMenu(PauseMenu),
}

impl GameState {
	pub fn new(graphics: &Graphics) -> Self {
		GameState::PlayingLevel(todo!())
	}

	pub fn input(
		&mut self,
		graphics: &mut Graphics,
		input: Input,
	) -> Result<(), String> {
		match self {
			GameState::PlayingLevel(level_player) => 
				level_player.input(graphics, input),
		}
	}

	pub fn render(
		&mut self, 
		surface: &mut impl Surface, 
		graphics: &mut Graphics,
		aspect: f32,
		dt: f32,
	) -> Result<(), String> {
		match self {
			GameState::PlayingLevel(level_player) => 
				level_player.render(surface, graphics, aspect, dt),
		}
	}
}

pub struct LevelPlayer {
	level_path: PathBuf,
	levels: Vec<PathBuf>,
	level: Level,
	level_graphics: LevelGraphics,
	cached_input: Option<Input>,
	events: VecDeque<Event>,
	hot_load_timer: f32,
	previous_load: std::time::SystemTime,
}

impl LevelPlayer {
	pub fn new(mut levels: Vec<PathBuf>, graphics: &mut Graphics) 
		-> Result<LevelPlayer, String> 
	{
		assert!(levels.len() > 0);
		let level_path = levels.remove(0);
		let level = Level::from_string(
			&fs::read_to_string(&level_path)
				.map_err(|v| v.to_string())?
		).map_err(|v| v.to_string())?;
		let level_graphics = LevelGraphics::new(graphics, &level);
		Ok(LevelPlayer {
			level_path,
			levels,
			level,
			level_graphics,
			cached_input: None,
			events: VecDeque::new(),
			hot_load_timer: 0.0,
			previous_load: std::time::SystemTime::now(),
		})
	}

	pub fn input(&mut self, graphics: &mut Graphics, input: Input) 
		-> Result<(), String> 
	{
		if input == Input::Restart {
			self.reload_level(graphics);
		}else if input == Input::Confirm && self.level.has_won {
			if self.levels.len() > 0 {
				self.level_path = self.levels.remove(0);
				match self.reload_level(graphics) {
					Ok(_) => (),
					Err(err) => println!("Error going to next level: {}", err),
				}
			}else {
				println!("No more levels!");
			}
		}else {
			self.cached_input = Some(input);
		}
		Ok(())
	}

	pub fn render(
		&mut self, 
		surface: &mut impl Surface, 
		graphics: &mut Graphics,
		aspect: f32,
		dt: f32,
	) -> Result<(), String> {
		self.hot_load_timer -= dt;
		if self.hot_load_timer < 0.0 {
			self.hot_load_timer = 1.0;
			if let Ok(Ok(new_time)) = 
				fs::metadata(&self.level_path).map(|v| v.modified())
			{
				if self.previous_load != new_time {
					match self.reload_level(graphics) {
						Ok(_) => (),
						Err(msg) => 
							println!("Cannot reload level because: {}", msg),
					}
				}
			} else {
				println!("Reading file metadata failed");
			}
		}

		if self.level_graphics.animations.len() == 0 {
			if let Some(input) = self.cached_input.take() {
				self.level.input(input, &mut self.events);
				for event in self.events.drain(..) {
					let time = 
						if let Event::EntityMoved { time_offset, .. } = event {
							-time_offset
						} else {
							0.0
						};
					self.level_graphics.animations.push_back((time, event));
				}
			}
		}

		surface.clear_color(0.3, 0.3, 0.5, 1.0);
		self.level_graphics
			.render_level(&graphics, surface, aspect, &mut self.level, dt);

		Ok(())
	}

	fn reload_level(&mut self, graphics: &mut Graphics) -> Result<(), String> {
		if let Ok(Ok(new_time)) = 
			fs::metadata(&self.level_path).map(|v| v.modified())
		{
			self.previous_load = new_time;
		}
		let level = Level::from_string(
			&fs::read_to_string(&self.level_path)
				.map_err(|v| v.to_string())?
		).map_err(|v| v.to_string())?;
		self.level_graphics
			.reset(graphics, &level);
		self.level = level;
		Ok(())
	}
}
