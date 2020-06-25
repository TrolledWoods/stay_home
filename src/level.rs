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
			active_events: Events::new(),
			old_events: None,
		})
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

		let mut is_kicked = false;
		match self.tiles[entity.x as usize + entity.y as usize * self.width] {
			Tile::Ice => is_kicked = true,
			_ => ()
		}

		self.active_events.moves.push(MoveEntity {
			is_kicked,
			can_ice_kick: true,
			slides_without_kick: true,
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

	pub fn update(&mut self, animations: &mut VecDeque<Animation>) {
		let mut events = std::mem::replace(
			&mut self.active_events, 
			self.old_events.take().unwrap_or_else(|| Events::new()),
		);
		let mut new_events = Events::new();

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

		let mut index = 0;
		while index < events.moves.len() {
			let move_ = events.moves[index];
			let to = move_.to();

			if let Some(id) = self.get_entity_at_tile(move_.to()) {
				match self.tiles[to[0] as usize + to[1] as usize * self.width] {
					Tile::Ice if move_.can_ice_kick => {
						animations.push_back(Animation::IceKick {
							entity_id: move_.entity_id,
							from: move_.from,
							to,
						});

						let entity = self.entities.get(&id).unwrap();
						events.moves.remove(index);
						new_events.moves.push(MoveEntity {
							can_ice_kick: true,
							is_kicked: true,
							..MoveEntity::new(id, [entity.x, entity.y], move_.direction)
						});
						continue;
					},
					_ => (),
				}

				let entity = self.entities.get(&id).unwrap();
				events.moves.push(MoveEntity {
					can_ice_kick: true,
					..MoveEntity::new(id, [entity.x, entity.y], move_.direction)
				});
			}

			index += 1;
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
				Tile::Ice if move_.is_kicked || move_.slides_without_kick => {
					new_events.moves.push(MoveEntity {
						can_ice_kick: move_.can_ice_kick,
						slides_without_kick: move_.slides_without_kick,
						is_kicked: true,
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
	can_ice_kick: bool,
	is_kicked: bool,
	slides_without_kick: bool,
	entity_id: u32,
	from: [isize; 2],
	direction: Input,
}

impl MoveEntity {
	fn new(entity_id: u32, from: [isize; 2], direction: Input) -> Self {
		MoveEntity {
			entity_id,
			from,
			direction,
			can_ice_kick: false,
			slides_without_kick: false,
			is_kicked: false,
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

#[derive(Clone, Copy)]
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
