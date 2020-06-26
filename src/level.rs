use crate::prelude::*;
use crate::textures::Texture;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Default)]
pub struct Level {
	pub n_humans: usize,

	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,

	pub active_events: Events,
	old_events: Option<Events>,

	pub n_tile_changes: u32,

	entity_id_ctr: u32,
	pub entities: HashMap<u32, Entity>,

	pub has_won: bool,

	player_id: u32,
}

impl Level {
	pub fn several_from_string(input: &str) -> Result<Vec<Level>, String> {
		let mut level: Level = Default::default();
		let mut levels = Vec::new();
		let mut making_level = false;
		let mut has_player = false;

		let lines = input.lines().rev().map(|v| v.trim());

		let mut y = 0;
		for line in lines {
			if line.len() == 0 || line.starts_with("//") {
				// If 
				if making_level {
					if !has_player {
						return Err(format!("Expected player"));
					}

					levels.insert(
						0,
						std::mem::replace(&mut level, Default::default())
					);
					making_level = false;
					has_player = false;
				}

				continue;
			}

			if making_level == false {
				y = 0;
			}

			making_level = true;

			if level.width == 0 { 
				level.width = line.len(); 
			}else if level.width != line.len() { 
				return Err(format!("Expected the same width for every line")); 
			}

			level.height += 1;

			for (x, char_) in line.chars().enumerate() {
				level.tiles.push(match char_ {
					// Entities
					'@' => {
						if has_player {
							return 
								Err(format!("Cannot have more than 1 player!"));
						}

						has_player = true;

						level.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Player));
						level.player_id = level.entity_id_ctr;
						level.entity_id_ctr += 1;
						Tile::Floor
					}
					'$' => {
						level.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Human));
						level.entity_id_ctr += 1;
						level.n_humans += 1;
						Tile::Floor
					}
					'B' => {
						level.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Block));
						level.entity_id_ctr += 1;
						Tile::Floor
					}
					'C' => {
						level.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Cake));
						level.entity_id_ctr += 1;
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

			y += 1;
		}

		if making_level {
			levels.insert(0, level);
		}

		Ok(levels)
	}

	pub fn input(&mut self, input: Input) {
		// @Cleanup: Direction enum!
		if input == Input::Confirm { return; }

		for move_ in self.active_events.moves.iter() {
			if move_.entity_id == self.player_id {
				println!("Cannot create a duplicate move!");
				return;
			}
		}

		let entity = self.entities.get(&self.player_id).unwrap();

		let is_friction_push = match self.get_tile(entity.x, entity.y).unwrap() {
			Tile::Ice => false,
			_ => true
		};

		self.active_events.moves.push(MoveEntity {
			is_friction_push,
			..MoveEntity::new(self.player_id, [entity.x, entity.y], input)
		});
	}

	pub fn tile_is_solid(&self, pos: [isize; 2]) -> bool {
		if pos[0] < 0 || pos[0] as usize >= self.width || 
			pos[1] < 0 || pos[1] as usize >= self.height 
		{
			return true;
		}

		match self.tiles[pos[0] as usize + pos[1] as usize * self.width] {
			Tile::Wall | Tile::HappyHome => return true,
			_ => (),
		}

		for entity in self.entities.values() {
			if entity.x == pos[0] && entity.y == pos[1] {
				return true;
			}
		}

		false
	}

	fn get_entity_at_tile(&self, pos: [isize; 2]) -> Option<u32> {
		for (&id, entity) in self.entities.iter() {
			if entity.x == pos[0] && entity.y == pos[1] {
				return Some(id);
			}
		}
		None
	}

	pub fn get_tile(&self, x: isize, y: isize) -> Option<Tile> {
		if x < 0 || x as usize >= self.width 
			|| y < 0 || y as usize >= self.height {
			return None;
		}

		// @Cleanup: get_unchecked?? Don't feel like doing unsafe for now.
		Some(self.tiles[x as usize + y as usize * self.width])
	}

	pub fn update(&mut self, animations: &mut VecDeque<Animation>) {
		let mut events = std::mem::replace(
			&mut self.active_events, 
			self.old_events.take().unwrap_or_else(|| Events::new()),
		);
		let mut new_events = Events::new();

		// Tile modification
		for tile_modification in events.tile_modifications.drain(..) {
			let at = tile_modification.at;
			self.tiles[at[0] as usize + at[1] as usize * self.width] =
				tile_modification.into;
			self.n_tile_changes += 1;

			animations.push_back(Animation::TileModification {
				entity_id: tile_modification.sacrifice,
				at,
				new_tile: tile_modification.into,
			});

			if let Some(entity) = 
				self.entities.remove(&tile_modification.sacrifice) 
			{
				if entity.kind == EntityKind::Human {
					self.n_humans -= 1;
					if self.n_humans == 0 {
						self.has_won = true;
					}
				}
			}
		}

		// Pushing things
		let mut index = 0;
		'outer: while index < events.moves.len() {
			let move_ = events.moves[index];
			let to = move_.to();
			if let Some(id) = self.get_entity_at_tile(to) {
				let one_self = self.entities.get(&move_.entity_id).unwrap();
				let entity = self.entities.get(&id).unwrap();

				// If the things in question is already moving out of the way,
				// increase the priority of that, and then move on!
				//
				// This feels a little bit hacky though, but it seems to work
				// fairly well with ice for now? I think this will break easily
				// if several different things are moving at once and interacting,
				// but that probably won't be a problem for now, at least not
				// with the current game mechanics.
				for (i, other_move) in events.moves.iter().enumerate() {
					if other_move.from == to {
						let other_move = events.moves.remove(i);
						events.moves.push(other_move);
						continue 'outer;
					}
				}
				
				match (
					self.get_tile(one_self.x, one_self.y).unwrap(),
					self.get_tile(entity.x,   entity.y  ).unwrap(),
				) {
					(_, Tile::Ice) if !move_.is_friction_push => {
						// If something isn't based on friction, and the target
						// is on ice, then transfer the energy, don't push!
						events.moves.remove(index);

						animations.push_back(Animation::IceKick {
							entity_id: move_.entity_id,
							from: [one_self.x, one_self.y],
							to:   [entity.x,   entity.y  ],
						});
						new_events.moves.push(MoveEntity::new(
							id,
							[entity.x, entity.y],
							move_.direction,
						));
					}
					(_, _) if !move_.is_friction_push => {
						// Cannot push things that are not on ice
						// while on ice.
						index += 1;
					}
					(_, _) => {
						// Just normal pushing
						events.moves.push(MoveEntity {
							is_friction_push: true,
							..MoveEntity::new(id, [entity.x, entity.y], move_.direction)
						});
						index += 1;
					}
				}
			} else {
				// No pushing!
				index += 1;
			}
		}

		// TODO: Resolve move conflicts

		// Run all the moves
		// It's run in reverse because the moves resulting from pushing
		// are always further back in the list, so if we reverse it those
		// are moved first, which allows the pushers to also be moved.
		for move_ in events.moves.drain(..).rev() {
			let to = move_.to();

			if self.tile_is_solid(to) {
				animations.push_back(Animation::FailedMove {
					entity_id: move_.entity_id,
					from: move_.from,
					to,
				});
				continue;
			}

			let entity = self.entities.get_mut(&move_.entity_id).unwrap();
			match self.tiles[to[0] as usize + to[1] as usize * self.width] {
				Tile::Ice => {
					new_events.moves.push(MoveEntity {
						..MoveEntity::new(move_.entity_id, to, move_.direction)
					});
				},
				Tile::SadHome if entity.kind == EntityKind::Cake => {
					new_events.tile_modifications.push(TileModification {
						into: Tile::Home,
						sacrifice: move_.entity_id,
						at: to,
					});
				}
				Tile::Home if entity.kind == EntityKind::Human => {
					new_events.tile_modifications.push(TileModification {
						into: Tile::HappyHome,
						sacrifice: move_.entity_id,
						at: to,
					});
				}
				_ => (),
			}

			entity.x = to[0];
			entity.y = to[1];

			animations.push_back(Animation::Move {
				entity_id: move_.entity_id,
				from: move_.from,
				to,
			});
		}

		self.active_events = new_events;
		self.old_events = Some(events);
	}
}

#[derive(Clone, Copy)]
pub enum Animation {
	Move				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	IceKick				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	FailedMove			{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	TileModification	{ entity_id: u32, at: [isize; 2], new_tile: Tile },
}

#[derive(Clone, Default)]
pub struct Events {
	pub moves: Vec<MoveEntity>,
	pub tile_modifications: Vec<TileModification>,
}

impl Events {
	fn new() -> Events {
		Events {
			moves: Vec::new(),
			tile_modifications: Vec::new(),
		}
	}

	pub fn empty(&self) -> bool {
		self.moves.len() == 0 && self.tile_modifications.len() == 0
	}
}

#[derive(Clone, Copy)]
pub struct MoveEntity {
	is_friction_push: bool,
	entity_id: u32,
	from: [isize; 2],
	direction: Input,
}

impl MoveEntity {
	fn new(entity_id: u32, from: [isize; 2], direction: Input) -> Self {
		MoveEntity {
			is_friction_push: false,
			entity_id,
			from,
			direction,
		}
	}
}

impl MoveEntity {
	pub fn to(&self) -> [isize; 2] {
		match self.direction {
			Input::MoveRight => [self.from[0] + 1, self.from[1]    ],
			Input::MoveUp    => [self.from[0]    , self.from[1] + 1],
			Input::MoveLeft  => [self.from[0] - 1, self.from[1]    ],
			Input::MoveDown  => [self.from[0]    , self.from[1] - 1],
			_ => todo!("Make a direction enum instead of an input enum"),
		}
	}
}

#[derive(Clone, Copy)]
pub struct TileModification {
	into: Tile,
	sacrifice: u32,
	at: [isize; 2],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	Floor,
	Wall,
	SadHome,
	Home,
	HappyHome,
	Ice,
}

#[derive(Clone, Copy)]
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
