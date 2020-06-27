use crate::prelude::*;
use crate::textures::Texture;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Default)]
pub struct Level {
	old_events: Option<Events>,

	pub n_tile_changes: u32,

	entity_id_ctr: u32,

	pub has_won: bool,

	pub data: LevelData,

	pub undo_stack: Vec<LevelData>,

	player_id: u32,
}

// All the data for a level state
#[derive(Clone, Default)]
pub struct LevelData {
	pub entities: HashMap<u32, Entity>,
	pub active_events: Events,
	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,
	pub n_humans: usize,
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

			if level.data.width == 0 { 
				level.data.width = line.len(); 
			}else if level.data.width != line.len() { 
				return Err(format!("Expected the same width for every line")); 
			}

			level.data.height += 1;

			for (x, char_) in line.chars().enumerate() {
				level.data.tiles.push(match char_ {
					// Entities
					'p' | 'P' => {
						if has_player {
							return 
								Err(format!("Cannot have more than 1 player!"));
						}

						has_player = true;

						level.data.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Player));
						level.player_id = level.entity_id_ctr;
						level.entity_id_ctr += 1;
						if char_.is_uppercase() { Tile::Ice } else { Tile::Floor }
					}
					'b' | 'B' => {
						level.data.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Human));
						level.entity_id_ctr += 1;
						level.data.n_humans += 1;
						if char_.is_uppercase() { Tile::Ice } else { Tile::Floor }
					}
					'c' | 'C' => {
						level.data.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::Cake));
						level.entity_id_ctr += 1;
						if char_.is_uppercase() { Tile::Ice } else { Tile::Floor }
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

		println!("Loaded {} levels", levels.len());

		Ok(levels)
	}

	pub fn input(&mut self, input: Input) {
		// @Cleanup: Direction enum!
		if input == Input::Confirm { return; }
		self.undo_stack.push(self.data.clone());

		for move_ in self.data.active_events.moves.iter() {
			if move_.entity_id == self.player_id {
				println!("Cannot create a duplicate move!");
				return;
			}
		}

		let entity = self.data.entities.get(&self.player_id).unwrap();

		let is_friction_push = match self.get_tile([entity.x, entity.y]).unwrap() {
			Tile::Ice => false,
			_ => true
		};

		self.data.active_events.moves.push(MoveEntity {
			is_friction_push,
			..MoveEntity::new(self.player_id, [entity.x, entity.y], input)
		});
	}

	pub fn tile_is_solid(&self, pos: [isize; 2]) -> bool {
		if pos[0] < 0 || pos[0] as usize >= self.data.width || 
			pos[1] < 0 || pos[1] as usize >= self.data.height 
		{
			return true;
		}

		match self.get_tile(pos).unwrap() {
			Tile::Wall | Tile::HappyHome => return true,
			_ => (),
		}

		for entity in self.data.entities.values() {
			if entity.x == pos[0] && entity.y == pos[1] {
				return true;
			}
		}

		false
	}

	fn get_entity_at_tile(&self, pos: [isize; 2]) -> Option<u32> {
		for (&id, entity) in self.data.entities.iter() {
			if entity.x == pos[0] && entity.y == pos[1] {
				return Some(id);
			}
		}
		None
	}

	pub fn get_tile(&self, pos: [isize; 2]) -> Option<Tile> {
		if pos[0] < 0 || pos[0] as usize >= self.data.width 
			|| pos[1] < 0 || pos[1] as usize >= self.data.height {
			return None;
		}

		// @Cleanup: get_unchecked?? Don't feel like doing unsafe for now.
		Some(self.data.tiles[pos[0] as usize + pos[1] as usize * self.data.width])
	}

	pub fn set_tile(&mut self, pos: [isize; 2], tile: Tile) -> bool {
		if pos[0] < 0 || pos[0] as usize >= self.data.width 
			|| pos[1] < 0 || pos[1] as usize >= self.data.height {
			return false;
		}

		// @Cleanup: get_unchecked?? Don't feel like doing unsafe for now.
		self.data.tiles[pos[0] as usize + pos[1] as usize * self.data.width] 
			= tile;
		true
	}

	pub fn update(&mut self, animations: &mut VecDeque<Animation>) {
		let mut events = std::mem::replace(
			&mut self.data.active_events, 
			self.old_events.take().unwrap_or_else(|| Events::new()),
		);
		let mut new_events = Events::new();

		// Tile modification
		for tile_modification in events.tile_modifications.drain(..) {
			let at = tile_modification.at;
			assert!(self.set_tile(at, tile_modification.into));
			self.n_tile_changes += 1;

			animations.push_back(Animation::TileModification {
				entity_id: tile_modification.sacrifice,
				at,
				new_tile: tile_modification.into,
			});

			if let Some(entity) = 
				self.data.entities.remove(&tile_modification.sacrifice) 
			{
				if entity.kind == EntityKind::Human {
					self.data.n_humans -= 1;
					if self.data.n_humans == 0 {
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
				let one_self = self.data.entities.get(&move_.entity_id).unwrap();
				let entity = self.data.entities.get(&id).unwrap();

				// If the things in question is already moving out of the way,
				// increase the priority of that, and then move on!
				//
				// @Cleanup: This method of doing things is silly. The
				// ``i == events.moves.len() - 1`` is just a hack to not make
				// it get stuck in an infinite loop.
				for (i, other_move) in events.moves.iter().enumerate() {
					if other_move.from == to {
						if i == events.moves.len() - 1 {
							// With the last move this is a noop?
							println!("The last move is having trouble");
							break;
						}

						let other_move = events.moves.remove(i);
						events.moves.push(other_move);
						continue 'outer;
					}
				}
				
				match (
					self.get_tile([one_self.x, one_self.y]).unwrap(),
					self.get_tile([entity.x,   entity.y  ]).unwrap(),
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
					// (_, _) if !move_.is_friction_push => {
					// 	// Cannot push things that are not on ice
					// 	// while on ice.
					// 	index += 1;
					// }
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

			let entity = self.data.entities.get_mut(&move_.entity_id).unwrap();
			match self.data.tiles[to[0] as usize + to[1] as usize * self.data.width] {
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

			if move_.is_friction_push {
				animations.push_back(Animation::Move {
					entity_id: move_.entity_id,
					from: move_.from,
					to,
				});
			}else {
				animations.push_back(Animation::IceSlide {
					entity_id: move_.entity_id,
					from: move_.from,
					to,
				});
			}
		}

		self.data.active_events = new_events;
		self.old_events = Some(events);
	}
}

#[derive(Clone, Copy)]
pub enum Animation {
	Move				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	IceKick				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	IceSlide            { entity_id: u32, from: [isize; 2], to: [isize; 2] },
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
	// @Cleanup: Coordinates with an array
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
