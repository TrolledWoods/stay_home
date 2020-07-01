use crate::prelude::*;
use crate::textures::Texture;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::sounds::{SoundId, Sounds};

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
	pub has_input: bool,
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
					'g' | 'G' => {
						level.data.entities.insert(level.entity_id_ctr, 
							Entity::new(x as isize, y as isize, EntityKind::BucketOfGoop));
						level.entity_id_ctr += 1;
						if char_.is_uppercase() { Tile::IceWithGoop } else { Tile::FloorWithGoop }
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

	pub fn randomized() -> Level {
		let width = 8;
		let height = 8;
		let n_house = 3;
		let n_sad_house = 2;
		let n_extra_cake = 4;
		let n_goops = 3;
		let n_wall = 7;
		let n_ice = 5;

		let mut level: Level = Default::default();
		level.data.width = width;
		level.data.height = height;
		level.data.tiles = vec![
			Tile::Floor; 
			level.data.width * level.data.height
		];

		let mut tiles = HashSet::new();
		for x in 0..level.data.width {
			for y in 0..level.data.height {
				tiles.insert([x as isize, y as isize]);
			}
		}
		let mut tiles = tiles.drain();

		for _ in 0..n_wall {
			level.set_tile(tiles.next().unwrap(), Tile::Wall);
		}

		for _ in 0..n_ice {
			level.set_tile(tiles.next().unwrap(), Tile::Ice);
		}
		for _ in 0..n_house {
			level.set_tile(tiles.next().unwrap(), Tile::Home);
		}
		for _ in 0..n_sad_house {
			level.set_tile(tiles.next().unwrap(), Tile::SadHome);
		}

		let old_tiles = tiles;
		let mut tiles = HashSet::new();
		for tile in old_tiles {
			if tile[0] == 0 || tile[1] == 0 
				|| tile[0] == width as isize - 1 || tile[1] == height as isize - 1 
			{
				continue;
			}

			// Cornders not allowed!
			if level.tile_is_solid([tile[0] - 1, tile[1]]) &&
				level.tile_is_solid([tile[0], tile[1] - 1]) {
				continue;
			}
			if level.tile_is_solid([tile[0] + 1, tile[1]]) &&
				level.tile_is_solid([tile[0], tile[1] - 1]) {
				continue;
			}
			if level.tile_is_solid([tile[0] - 1, tile[1]]) &&
				level.tile_is_solid([tile[0], tile[1] + 1]) {
				continue;
			}
			if level.tile_is_solid([tile[0] + 1, tile[1]]) &&
				level.tile_is_solid([tile[0], tile[1] + 1]) {
				continue;
			}

			tiles.insert(tile);
		}
		let mut tiles = tiles.drain();

		// Player!
		let [x, y] = tiles.next().unwrap();
		level.data.entities.insert(level.entity_id_ctr, 
			Entity::new(x, y, EntityKind::Player));
		level.player_id = level.entity_id_ctr;
		level.entity_id_ctr += 1;

		for _ in 0..n_goops {
			let [x, y] = tiles.next().unwrap();
			level.data.entities.insert(level.entity_id_ctr, 
				Entity::new(x, y, EntityKind::BucketOfGoop));
			level.set_tile([x, y], Tile::FloorWithGoop);
			level.entity_id_ctr += 1;
		}

		// Cakes
		for _ in 0..(n_sad_house + n_extra_cake) {
			let [x, y] = tiles.next().unwrap();
			level.data.entities.insert(level.entity_id_ctr, 
				Entity::new(x, y, EntityKind::Cake));
			level.entity_id_ctr += 1;
		}

		// Humans
		for _ in 0..(n_house + n_sad_house) {
			let [x, y] = tiles.next().unwrap();
			level.data.entities.insert(level.entity_id_ctr, 
				Entity::new(x, y, EntityKind::Human));
			level.entity_id_ctr += 1;
			level.data.n_humans += 1;
		}

		level
	}

	pub fn input(&mut self, input: Direction) {
		for move_ in self.data.active_events.moves.iter() {
			if move_.entity_id == self.player_id {
				return;
			}
		}

		self.data.has_input = true;

		let entity = self.data.entities.get(&self.player_id).unwrap();

		let is_friction_push = match self.get_tile([entity.x, entity.y]).unwrap() {
			Tile::Ice => false,
			_ => true
		};

		let move_ = MoveEntity {
			is_friction_push,
			..MoveEntity::new(self.player_id, [entity.x, entity.y], input)
		};

		// TODO: Only add an undo state when something actually happens.
		self.undo_stack.push(self.data.clone());

		self.data.active_events.moves.push(move_);
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

	pub fn update(&mut self, animations: &mut VecDeque<Animation>, sounds: &Sounds) {
		let mut events = std::mem::replace(
			&mut self.data.active_events, 
			self.old_events.take().unwrap_or_else(|| Events::new()),
		);
		let mut new_events = Events::new();

		// Pushing things
		let mut index = 0;
		let mut pushing_happened = false;
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

				if (one_self.kind == EntityKind::HumanWithGoop && entity.kind == EntityKind::Cake) ||
					(one_self.kind == EntityKind::Cake && entity.kind == EntityKind::HumanWithGoop) 
				{
					// The goop child eats the cake!
					let other = self.data.entities.get_mut(&id).unwrap();
					other.kind = EntityKind::Human;
					let other_pos = [other.x, other.y];
					let me = self.data.entities.get_mut(&move_.entity_id).unwrap();
					let me_pos = [me.x, me.y];
					animations.push_back(Animation::Eat {
						from: me_pos,
						to: other_pos,
						eater: id,
						eating: move_.entity_id,
					});
					animations.push_back(Animation::Goopify { entity_id: id, kind: EntityKind::Human });
					events.moves.remove(index);
					self.data.entities.remove(&move_.entity_id);
					continue;
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
						let move_ = MoveEntity::new(
							id,
							[entity.x, entity.y],
							move_.direction,
						);
						new_events.moves.push(move_);
					}
					// (_, _) if !move_.is_friction_push => {
					// 	// Cannot push things that are not on ice
					// 	// while on ice.
					// 	index += 1;
					// }
					(_, _) => {
						// Just normal pushing
						let move_ = MoveEntity {
							is_friction_push: true,
							..MoveEntity::new(id, [entity.x, entity.y], move_.direction)
						};
						events.moves.push(move_);
						index += 1;
						pushing_happened = true;
					}
				};

			} else {
				// No pushing!
				index += 1;
			}
		}

		if pushing_happened {
			sounds.play(SoundId::Push, 0.1);
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

			let entity = self.data.entities.get(&move_.entity_id).unwrap();
			if entity.kind == EntityKind::BucketOfGoop {
				match self.get_tile(move_.to()).unwrap() {
					Tile::Ice => {
						self.set_tile(move_.to(), Tile::IceWithGoop);
						self.n_tile_changes += 1;
					}
					Tile::Floor => {
						self.set_tile(move_.to(), Tile::FloorWithGoop);
						self.n_tile_changes += 1;
					}
					Tile::Home | Tile::SadHome => {
						// Cannot move Bucket of Goop onto houses.
						animations.push_back(Animation::FailedMove {
							entity_id: move_.entity_id,
							from: move_.from,
							to,
						});
						continue;
					}
					_ => (),
				}
			}

			let entity = self.data.entities.get_mut(&move_.entity_id).unwrap();
			match self.data.tiles[to[0] as usize + to[1] as usize * self.data.width] {
				Tile::Ice => {
					new_events.moves.push(MoveEntity {
						..MoveEntity::new(move_.entity_id, to, move_.direction)
					});
				},
				Tile::IceWithGoop => {
					// new_events.moves.push(MoveEntity {
					// 	..MoveEntity::new(move_.entity_id, to, move_.direction)
					// });
					entity.goopify();
					animations.push_back(Animation::Goopify { entity_id: move_.entity_id, kind: entity.kind });
				}
				Tile::FloorWithGoop => {
					entity.goopify();
					animations.push_back(Animation::Goopify { entity_id: move_.entity_id, kind: entity.kind });
				},
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

		// Tile modification
		let mut entities_to_remove = Vec::new();
		for (&entity_id, entity) in self.data.entities.iter() {
			match (entity.kind, self.data.tiles[entity.x as usize + entity.y as usize * self.data.width]) {
				(EntityKind::Human, Tile::Home) => {
					self.data.tiles[entity.x as usize + entity.y as usize * self.data.width] = Tile::HappyHome;
					self.n_tile_changes += 1;
					animations.push_back(Animation::TileModification {
						entity_id,
						at: [entity.x, entity.y],
						new_tile: Tile::HappyHome,
					});
					self.data.n_humans -= 1;
					if self.data.n_humans == 0 {
						self.has_won = true;
					}
					entities_to_remove.push(entity_id);
				}
				(EntityKind::Cake, Tile::SadHome) => {
					self.data.tiles[entity.x as usize + entity.y as usize * self.data.width] = Tile::Home;
					self.n_tile_changes += 1;
					animations.push_back(Animation::TileModification {
						entity_id,
						at: [entity.x, entity.y],
						new_tile: Tile::Home,
					});
					entities_to_remove.push(entity_id);
				}
				_ => (),
			}
		}

		for entity in entities_to_remove {
			self.data.entities.remove(&entity);
		}

		if self.data.has_input {
			sounds.play(SoundId::SpiderWalk, 0.1);
			self.data.has_input = false;
		}

		self.data.active_events = new_events;
		self.old_events = Some(events);
	}
}

#[derive(Clone, Copy)]
pub enum Animation {
	// @Cleanup: Unify / Remove some of these, it feels as if some of
	// them are redundant, or are used in contexts where they shouldn't.
	Goopify				{ entity_id: u32, kind: EntityKind },
	Eat {
		from: [isize; 2],
		to: [isize; 2],
		eating: u32,
		eater: u32,
	},
	Move				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	IceKick				{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	IceSlide            { entity_id: u32, from: [isize; 2], to: [isize; 2] },
	FailedMove			{ entity_id: u32, from: [isize; 2], to: [isize; 2] },
	TileModification	{ entity_id: u32, at: [isize; 2], new_tile: Tile },
}

#[derive(Clone, Default)]
pub struct Events {
	pub moves: Vec<MoveEntity>,
}

impl Events {
	fn new() -> Events {
		Events {
			moves: Vec::new(),
		}
	}

	pub fn empty(&self) -> bool {
		self.moves.len() == 0
	}
}

#[derive(Clone, Copy)]
pub struct MoveEntity {
	is_friction_push: bool,
	entity_id: u32,
	from: [isize; 2],
	direction: Direction,
}

impl MoveEntity {
	fn new(entity_id: u32, from: [isize; 2], direction: Direction) -> Self {
		MoveEntity {
			is_friction_push: false,
			entity_id,
			from,
			direction,
		}
	}

	pub fn to(&self) -> [isize; 2] {
		match self.direction {
			Direction::Right => [self.from[0] + 1, self.from[1]    ],
			Direction::Up    => [self.from[0]    , self.from[1] + 1],
			Direction::Left  => [self.from[0] - 1, self.from[1]    ],
			Direction::Down  => [self.from[0]    , self.from[1] - 1],
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	Floor,
	Wall,
	SadHome,
	Home,
	HappyHome,
	Ice,
	FloorWithGoop,
	IceWithGoop,
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

	pub fn goopify(&mut self) {
		match self.kind {
			EntityKind::Cake => self.kind = EntityKind::CakeWithGoop,
			EntityKind::Human => self.kind = EntityKind::HumanWithGoop,
			_ => (),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
	Player,
	Human,
	Cake,
	BucketOfGoop,
	HumanWithGoop,
	CakeWithGoop,
}

impl EntityKind {
	pub fn get_texture(&self) -> Texture {
		match self {
			EntityKind::Player => Texture::Player,
			EntityKind::Human => Texture::Human,
			EntityKind::Cake => Texture::Cake,
			EntityKind::BucketOfGoop => Texture::BucketOfGoop,
			EntityKind::HumanWithGoop => Texture::HumanWithGoop,
			EntityKind::CakeWithGoop => Texture::CakeWithGoop,
		}
	}
}
