use crate::prelude::*;
use std::path::PathBuf;
use std::fs;

pub enum GameState {
	// MainMenu(MainMenu),
	PlayingLevel(LevelPlayer),
	// PauseMenu(PauseMenu),
}

impl GameState {
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
	levels: Vec<Level>,
	current_level: usize,
	level: Level,
	level_graphics: LevelGraphics,
	cached_input: Option<Input>,
	hot_load_timer: f32,
	previous_load: std::time::SystemTime,
	update_timer: f32,
}

impl LevelPlayer {
	pub fn new(level_path: PathBuf, graphics: &mut Graphics) 
		-> Result<LevelPlayer, String> 
	{
		let levels = Level::several_from_string(
			&fs::read_to_string(&level_path)
				.map_err(|v| v.to_string())?
		).map_err(|v| v.to_string())?;
		if levels.len() == 0 {
			return Err(format!("You have to have at least 1 level!"));
		}
		// Levels go in the opposite direction
		let level = levels[0].clone();
		let level_graphics = LevelGraphics::new(graphics, &level);
		Ok(LevelPlayer {
			level_path,
			current_level: 0,
			levels,
			level,
			level_graphics,
			cached_input: None,
			hot_load_timer: 0.0,
			previous_load: std::time::SystemTime::now(),
			update_timer: 0.0,
		})
	}

	pub fn input(&mut self, graphics: &mut Graphics, input: Input) 
		-> Result<(), String> 
	{
		// @Cleanup: Use a friggin match
		let mut level_changed = false;
		if input == Input::PrevLevel {
			if self.current_level > 0 {
				self.current_level -= 1;
			}else {
				println!("No previous level");
			}
			level_changed = true;
		}
		if input == Input::NextLevel || 
			(input == Input::Confirm && self.level.has_won)
		{
			if self.current_level < self.levels.len() - 1 {
				self.current_level += 1;
			}else {
				println!("No more levels!");
			}
			level_changed = true;
		}
		if input == Input::Restart || 
			(input == Input::Confirm && !self.level.has_won) 
		{
			level_changed = true;
		}

		if level_changed {
			match self.reload_level(graphics) {
				Ok(_) => (),
				Err(err) => println!("Error going to next level: {}", err),
			}

			return Ok(());
		}

		self.cached_input = Some(input);
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
					self.previous_load = new_time;
					let levels = Level::several_from_string(
						&fs::read_to_string(&self.level_path)
						.map_err(|v| v.to_string())?
					).map_err(|v| v.to_string())?;
					if levels.len() == 0 {
						return Err(format!("You have to have at least 1 level!"));
					}
					self.levels = levels;

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

		self.update_timer -= dt * 7.0;
		if self.update_timer <= 0.0 {
			self.level_graphics.animations.clear();

			if let Some(input) = self.cached_input.take() {
				self.level.input(input);
			}

			if !self.level.data.active_events.empty() {
				self.update_timer = 1.0;
				self.level.update(&mut self.level_graphics.animations);
			}
		}

		surface.clear_color(0.3, 0.3, 0.5, 1.0);
		self.level_graphics.render_level(
			&graphics, 
			surface, 
			aspect, 
			&mut self.level, 
			1.0f32.min(1.0 - self.update_timer),
			dt,
		);

		Ok(())
	}

	fn reload_level(&mut self, graphics: &mut Graphics) -> Result<(), String> {
		let level = self.levels[self.current_level].clone();
		self.level_graphics
			.reset(graphics, &level);
		self.level = level;
		Ok(())
	}
}
