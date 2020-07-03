use crate::prelude::*;
use crate::level_graphics::smooth_lerp_time;
use crate::textures::UVCoords;
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
	// @Cleanup: Make a cached LevelGraphics to cache the levelgraphics.
	next_level_graphics: Option<(f32, LevelGraphics, Level, [f32; 2])>,
	cached_input: Option<Direction>,
	hot_load_timer: f32,
	previous_load: std::time::SystemTime,
	update_timer: f32,
	time: f32,
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
			next_level_graphics: None,
			cached_input: None,
			hot_load_timer: 0.0,
			previous_load: std::time::SystemTime::now(),
			update_timer: 0.0,
			time: 0.0,
		})
	}

	pub fn input(&mut self, graphics: &mut Graphics, input: Input) 
		-> Result<(), String> 
	{
		match input {
			Input::Randomize => {
				self.reload_level_dramatic(
					graphics,
					Level::randomized(),
					[1.0, 0.0],
				);
			}
			Input::Undo => {
				if let Some(undo_state) = self.level.undo_stack.pop() {
					self.level.data = undo_state;
					self.level_graphics
						.reset(graphics, &self.level);
				}else {
					println!("Nothing to undo!");
				}
			}
			Input::PrevLevel => {
				if self.current_level > 0 {
					self.current_level -= 1;
					self.reload_level_dramatic(
						graphics, 
						self.levels[self.current_level].clone(),
						[0.0, -1.0],
					);
				}else {
					println!("No previous level");
				}
			}
			Input::Confirm if self.level.has_won => {
				if self.current_level < self.levels.len() - 1 {
					self.current_level += 1;
					self.reload_level_dramatic(
						graphics, 
						self.levels[self.current_level].clone(),
						[0.0, 1.0],
					);
				}else {
					println!("No more levels!");
				}
			}
			Input::Confirm => {
				self.reload_level(
					graphics, 
					self.levels[self.current_level].clone(),
				);
			}
			Input::NextLevel => {
				if self.current_level < self.levels.len() - 1 {
					self.current_level += 1;
					self.reload_level_dramatic(
						graphics,
						self.levels[self.current_level].clone(),
						[0.0, 1.0],
					);
				}else {
					println!("No more levels!");
				}
			}
			Input::Move(direction) => {
				self.cached_input = Some(direction);
			}
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
		self.time += dt;

		if self.level.has_won {
			if self.current_level < self.levels.len() - 1 {
				self.current_level += 1;
				self.reload_level_dramatic(
					graphics, 
					self.levels[self.current_level].clone(),
					[0.0, 1.0],
				);
			}else {
				println!("No more levels!");
				self.current_level = 0;
				self.reload_level_dramatic(
					graphics, 
					self.levels[self.current_level].clone(),
					[0.0, 1.0],
				);
			}
		}

		self.hot_load_timer -= dt;
		if self.hot_load_timer < 0.0 {
			self.hot_load_timer = 1.0;
			if let Ok(Ok(new_time)) = 
				fs::metadata(&self.level_path).map(|v| v.modified())
			{
				if self.previous_load != new_time {
					self.previous_load = new_time;
					// @Cleanup: Catch the error here, don't let it fall through!
					// We don't wanna crash when the levels are invalid
					let levels = Level::several_from_string(
						&fs::read_to_string(&self.level_path)
						.map_err(|v| v.to_string())?
					).map_err(|v| v.to_string())?;
					if levels.len() == 0 {
						return Err(format!("You have to have at least 1 level!"));
					}
					self.levels = levels;

					self.reload_level(
						graphics, 
						self.levels[self.current_level].clone()
					);
				}
			} else {
				println!("Reading file metadata failed");
			}
		}

		graphics.draw_background_immediate(
			surface,
			aspect,
			[-1.0, -1.0, 2.0, 2.0],
			UVCoords {
				left: -100.0 * aspect,
				right: 100.0 * aspect,
				bottom: -100.0,
				top: 100.0,
			},
			self.time
		);

		if let Some((ref mut time, ref mut next_level_graphics, ref mut prev_level, [move_x, move_y])) 
			= self.next_level_graphics 
		{
			// Do the "dramatic next level animation"
			*time -= dt * 1.0;

			// Update the previous graphics, but slowly
			self.update_timer -= dt * lerp(0.0, 6.0, *time);

			let offset = smooth_lerp_time(*time, true, true);
			self.level_graphics.render_level(
				&graphics, 
				surface, 
				aspect, 
				prev_level, 
				[move_x * (offset - 1.0), move_y * (offset - 1.0)],
				1.0f32.min(1.0 - self.update_timer),
			);
			next_level_graphics.render_level(
				&graphics, 
				surface, 
				aspect, 
				&mut self.level, 
				[move_x * offset, move_y * offset],
				0.0,
			);

			if *time < 0.0 {
				self.level_graphics = self.next_level_graphics.take().unwrap().1;
			}
			return Ok(());
		}

		self.update_timer -= dt * 6.0;

		self.level_graphics.render_level(
			&graphics, 
			surface, 
			aspect, 
			&mut self.level, 
			[0.0, 0.0],
			1.0f32.min(1.0 - self.update_timer),
		);

		if self.update_timer <= 0.0 {
			self.level_graphics.animations.clear();

			if let Some(input) = self.cached_input.take() {
				self.level.input(input);
			}

			if !self.level.data.active_events.empty() {
				self.update_timer = 1.0;
				self.level.update(&mut self.level_graphics.animations, &graphics.sounds);
			}
		}

		Ok(())
	}

	fn reload_level(&mut self, graphics: &mut Graphics, level: Level) {
		if let Some((_, gfx, _, _)) = self.next_level_graphics.take() {
			self.level_graphics = gfx;
		}

		self.level = level;
		self.level_graphics.reset(graphics, &self.level);
	}

	fn reload_level_dramatic(&mut self, graphics: &mut Graphics, level: Level, direction: [f32; 2]) {
		if let Some((_, gfx, _, _)) = self.next_level_graphics.take() {
			self.level_graphics = gfx;
		}

		let old = std::mem::replace(
			&mut self.level,
			level,
		);
		self.next_level_graphics
			= Some((1.0, LevelGraphics::new(graphics, &self.level), old, direction));
	}
}
