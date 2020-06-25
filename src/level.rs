use crate::prelude::*;
use crate::textures::Texture;
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub struct Level {
	pub n_humans: usize,

	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,

	pub n_tile_changes: u32,

	entity_id_ctr: u32,
	pub entities: HashMap<u32, Entity>,

	pub has_won: bool,

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

		let lines = input.lines().map(|v| v.trim()).take_while(|v| !v.is_empty()).collect::<Vec<_>>();
		for (y, line) in lines.into_iter().rev().enumerate()
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
							Entity::new(x as isize, y as isize, EntityKind::Player));
						player_id = Some(entity_id_ctr);
						entity_id_ctr += 1;
						Tile::Floor
					}
					'$' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Human));
						entity_id_ctr += 1;
						n_humans += 1;
						Tile::Floor
					}
					'B' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Block));
						entity_id_ctr += 1;
						Tile::Floor
					}
					'C' => {
						entities.insert(entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Cake));
						entity_id_ctr += 1;
						Tile::Floor
					}
					'.' => Tile::Floor,
					'#' => Tile::Wall,
					'H' => Tile::Home,
					'S' => Tile::SadHome,
					'%' => Tile::Ice,
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
			has_won: false,
			n_tile_changes: 0,
		})
	}

	pub fn input(&mut self, input: Input, events: &mut VecDeque<Event>) {
		self.move_entity(0.0, self.player_id, input, events);
	}

	fn move_entity(&mut self, time_offset: f32, id: u32, input: Input, events: &mut VecDeque<Event>) -> bool {
		// @Cleanup: For now we expect the entity is not dead...
		let mut entity = self.entities.get(&id).unwrap();
		let old_x = entity.x;
		let old_y = entity.y;
		let mut x = entity.x;
		let mut y = entity.y;
		match input {
			Input::MoveLeft => {
				x -= 1;
			},
			Input::MoveRight => {
				x += 1;
			},
			Input::MoveDown => {
				y -= 1;
			},
			Input::MoveUp => {
				y += 1;
			},
			Input::Confirm => (),
			Input::Restart => (),
		}

		if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
			events.push_back(Event::MoveFailure {
				entity_id: id,
				from: [old_x, old_y],
				to: [x, y],
			});
			return false;
		}

		if self.tiles[x as usize + y as usize * self.width].is_solid() {
			events.push_back(Event::MoveFailure {
				entity_id: id,
				from: [old_x, old_y],
				to: [x, y],
			});
			return false;
		}

		let mut moving_into = None;
		for (&other_id, entity) in self.entities.iter() {
			if other_id == id { continue; }

			if entity.x == x && entity.y == y {
				moving_into = Some(other_id);
				break;
			}
		}


		if let Some(moving_into) = moving_into {
			// If that entity couldn't move, we can't either!
			if !self.move_entity(time_offset, moving_into, input, events) {
				events.push_back(Event::MoveFailure {
					entity_id: id,
					from: [old_x, old_y],
					to: [x, y],
				});
				return false;
			}
		}

		if self.tiles[x as usize + y as usize * self.width] == Tile::Ice {
			events.push_back(Event::EntityMoved {
				time_offset,
				entity_id: id,
				from: [old_x, old_y],
				to: [x, y],
			});
			let entity = self.entities.get_mut(&id).unwrap();
			entity.x = x;
			entity.y = y;
			self.move_entity(time_offset + 1.0, id, input, events);
			return true;
		}

		let entity = self.entities.get(&id).unwrap();
		if entity.kind == EntityKind::Cake && 
			self.tiles[x as usize + y as usize * self.width] == Tile::SadHome
		{
			self.tiles[x as usize + y as usize * self.width] = Tile::Home;
			self.n_tile_changes += 1;

			self.entities.remove(&id);
			events.push_back(Event::HomeSatisfied {
				home_loc: [x, y],
				from: [old_x, old_y],
				satisfier: id,
			});
			return true;
		}

		if entity.kind == EntityKind::Human && 
			self.tiles[x as usize + y as usize * self.width] == Tile::Home
		{
			self.tiles[x as usize + y as usize * self.width] = Tile::HappyHome;
			self.n_tile_changes += 1;

			self.n_humans -= 1;
			if self.n_humans == 0 {
				self.has_won = true;
			}

			self.entities.remove(&id);
			events.push_back(Event::HomeSatisfied {
				home_loc: [x, y],
				from: [old_x, old_y],
				satisfier: id,
			});
			return true;
		}

		println!("Entity {} moved!", id);
		events.push_back(Event::EntityMoved {
			time_offset,
			entity_id: id,
			from: [old_x, old_y],
			to: [x, y],
		});

		let entity = self.entities.get_mut(&id).unwrap();
		entity.x = x;
		entity.y = y;

		true
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Event {
	EntityMoved {
		time_offset: f32,
		entity_id: u32,
		from: [isize; 2],
		to: [isize; 2],
	},
	// TODO: Make this just "EntityUsedOnTile" or something
	HomeSatisfied {
		home_loc: [isize; 2],
		satisfier: u32,
		from: [isize; 2],
	},
	MoveFailure {
		entity_id: u32,
		from: [isize; 2],
		to: [isize; 2],
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
	Home,
	HappyHome,
	SadHome,
	Floor,
	Wall,
	Ice,
}

impl Tile {
	pub fn is_solid(&self) -> bool {
		use Tile::*;
		match self {
			Wall | HappyHome => true,
			_ => false,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Entity {
	pub x: isize,
	pub y: isize,
	pub kind: EntityKind,
}

impl Entity {
	pub fn new(x: isize, y: isize, kind: EntityKind) -> Self {
		Entity { x, y, kind }
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
	Player,
	Human,
	Block,
	Cake,
}

impl EntityKind {
	pub fn get_texture(&self) -> Texture {
		match self {
			EntityKind::Player => Texture::Player,
			EntityKind::Human => Texture::Human,
			EntityKind::Cake => Texture::Cake,
			EntityKind::Block => Texture::Wall,
		}
	}
}
