use crate::prelude::*;
use std::collections::{HashMap, VecDeque};

pub struct Level {
	pub n_humans: usize,

	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,

	entity_id_ctr: u32,
	pub entities: HashMap<u32, Entity>,

	player_id: u32,
}

impl Level {
	pub fn from_string(input: &str) -> Result<Level, String> {
		let mut n_humans = 0;
		let mut tiles = Vec::new();
		let mut entities = HashMap::new();
		let mut entity_id_ctr = 0;
		let mut width = 0;
		let mut height = 0;

		let mut player_id = None;

		for (y, line) in input.lines()
			.map(|v| v.trim())
			.filter(|v| !v.is_empty()) 
			.enumerate()
		{
			if width == 0 { 
				width = line.len(); 
			}else if width != line.len() { 
				return Err(format!("Expected the same width for every line")); 
			}

			height += 1;

			for (x, char_) in line.chars().enumerate() {
				tiles.push(match char_ {
					// Entities
					'@' => {
						if player_id.is_some() {
							return 
								Err(format!("Cannot have more than 1 player!"));
						}

						entities.insert(entity_id_ctr, 
							Entity::new(x, y, EntityKind::Human));
						player_id = Some(entity_id_ctr);
						entity_id_ctr += 1;
						n_humans += 1;
						Tile::Floor
					}
					'$' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x, y, EntityKind::Human));
						entity_id_ctr += 1;
						n_humans += 1;
						Tile::Floor
					}
					'B' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x, y, EntityKind::Block));
						entity_id_ctr += 1;
						Tile::Floor
					}
					'C' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x, y, EntityKind::Cake));
						entity_id_ctr += 1;
						Tile::Floor
					}

					'.' => Tile::Floor,
					'#' => Tile::Wall,
					'H' => Tile::Home,
					c => return Err(format!("Unknown character {}", c)),
				});
			}
		}

		let player_id = player_id.ok_or_else(|| format!("Expected a player"))?;
		Ok(Level {
			n_humans,
			width,
			height,
			tiles,
			entity_id_ctr,
			entities,
			player_id,
		})
	}

	pub fn input(&mut self, input: Input, events: &mut VecDeque<Event>) {
		// @Cleanup: For now we expect the player is not dead...
		let mut player = self.entities.get_mut(&self.player_id).unwrap();
		let old_x = player.x;
		let old_y = player.y;
		match input {
			Input::MoveLeft => if player.x > 0 {
				player.x -= 1;
			},
			Input::MoveRight => if player.x < self.width - 1 {
				player.x += 1;
			},
			Input::MoveDown => if player.y > 0 {
				player.y -= 1;
			},
			Input::MoveUp => if player.y < self.height - 1 {
				player.y += 1;
			},
			Input::Confirm => (),
		}

		if player.x != old_x || player.y != old_y {
			println!("Player moved!");
			events.push_back(Event::EntityMoved {
				entity_id: self.player_id,
				from: [old_x, old_y],
				to: [player.x, player.y],
			});
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Event {
	EntityMoved {
		entity_id: u32,
		from: [usize; 2],
		to: [usize; 2],
	},
	// TODO: Cause of death included for animation purposes?
	EntityDeath(u32),
	HomeSatisfied {
		home_loc: [usize; 2],
		satisfier: u32,
	}
}

#[derive(Clone, Copy, Debug)]
pub enum Tile {
	Home,
	Floor,
	Wall,
}

impl Tile {
	pub fn id(&self) -> usize {
		use Tile::*;
		match self {
			Home => 0,
			Floor => 1,
			Wall => 2,
		}
	}

	pub fn n_tiles() -> usize { 3 }
}

#[derive(Clone, Copy, Debug)]
pub struct Entity {
	pub x: usize,
	pub y: usize,
	pub kind: EntityKind,
}

impl Entity {
	pub fn new(x: usize, y: usize, kind: EntityKind) -> Self {
		Entity { x, y, kind }
	}
}

#[derive(Clone, Copy, Debug)]
pub enum EntityKind {
	Human,
	Block,
	Cake,
}
