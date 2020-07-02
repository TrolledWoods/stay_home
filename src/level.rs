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
	pub tiles: Tilemap,
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

			if level.width() == 0 { 
				level.data.tiles.width = line.len(); 
			}else if level.width() != line.len() { 
				return Err(format!("Expected the same width for every line")); 
			}

			level.data.tiles.height += 1;

			for (x, char_) in line.chars().enumerate() {
				level.data.tiles.buffer.push(match char_ {
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
					'#' => Tile::Wall(WallKind::Void),
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
		level.data.tiles.width = width;
		level.data.tiles.height = height;
		level.data.tiles.buffer = vec![Tile::Floor; width * height];

		let mut tiles = HashSet::new();
		for x in 0..width {
			for y in 0..height {
				tiles.insert([x as isize, y as isize]);
			}
		}
		let mut tiles = tiles.drain();

		for _ in 0..n_wall {
			level.data.tiles.set_tile(
				tiles.next().unwrap(), 
				Tile::Wall(WallKind::Void),
			);
		}

		for _ in 0..n_ice {
			level.data.tiles.set_tile(
				tiles.next().unwrap(),
				Tile::Ice,
			);
		}
		for _ in 0..n_house {
			level.data.tiles.set_tile(
				tiles.next().unwrap(),
				Tile::Home,
			);
		}
		for _ in 0..n_sad_house {
			level.data.tiles.set_tile(
				tiles.next().unwrap(),
				Tile::SadHome,
			);
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
			level.data.tiles.set_tile([x, y], Tile::FloorWithGoop);
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

	#[inline]
	pub fn width(&self) -> usize {
		self.data.tiles.width
	}

	#[inline]
	pub fn height(&self) -> usize {
		self.data.tiles.height
	}

	pub fn input(&mut self, input: Direction) {
		for move_ in self.data.active_events.moves.iter() {
			if move_.entity_id == self.player_id {
				return;
			}
		}

		self.data.has_input = true;

		let entity = self.data.entities.get(&self.player_id).unwrap();

		let is_friction_push = 
			match self.data.tiles.get_tile(entity.pos).unwrap() {
				Tile::Ice => false,
				_ => true
			};

		let move_ = MoveEntity {
			is_friction_push,
			..MoveEntity::new(self.player_id, entity.pos, input)
		};

		// TODO: Only add an undo state when something actually happens.
		self.undo_stack.push(self.data.clone());

		self.data.active_events.moves.push(move_);
	}

	pub fn tile_is_solid(&self, pos: [isize; 2]) -> bool {
		if pos[0] < 0 || pos[0] as usize >= self.width() || 
			pos[1] < 0 || pos[1] as usize >= self.height()
		{
			return true;
		}

		match self.data.tiles.get_tile(pos).unwrap() {
			Tile::Wall(_) => return true,
			_ => (),
		}

		for entity in self.data.entities.values() {
			if entity.pos == pos {
				return true;
			}
		}

		false
	}

	fn get_entity_at_tile(&self, pos: [isize; 2]) -> Option<u32> {
		for (&id, entity) in self.data.entities.iter() {
			if entity.pos == pos {
				return Some(id);
			}
		}
		None
	}

	// @Cleanup: Remove the Sounds import here, and instead cycle through the 
	// animations and apply sounds that way.
	// @Cleanup: Make undo only save states where you move and push 
	// something/slide on ice.
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
					let other_pos = other.pos;
					let me = self.data.entities.get_mut(&move_.entity_id).unwrap();
					let me_pos = me.pos;
					animations.push_back(Animation::Move {
						from: me_pos,
						to: other_pos,
						entity_id: move_.entity_id,
						accelerate: !me.is_sliding,
						decelerate: true,
						kind: AnimationMoveKind::Apply,
					});
					animations.push_back(Animation::Goopify { entity_id: id, kind: EntityKind::Human });
					events.moves.remove(index);
					self.data.entities.remove(&move_.entity_id);
					continue;
				}
				
				match (
					self.data.tiles.get_tile(one_self.pos).unwrap(),
					self.data.tiles.get_tile(entity.pos).unwrap(),
				) {
					(_, Tile::Ice) if !move_.is_friction_push => {
						// If something isn't based on friction, and the target
						// is on ice, then transfer the energy, don't push!
						events.moves.remove(index);

						animations.push_back(Animation::Move {
							entity_id: move_.entity_id,
							from: one_self.pos,
							to:   entity.pos,
							accelerate: !one_self.is_sliding,
							decelerate: true,
							kind: AnimationMoveKind::IceKick,
						});
						let move_ = MoveEntity::new(
							id,
							entity.pos,
							move_.direction,
						);
						events.moves.push(move_);
					}
					(_, _) => {
						// Just normal pushing
						let move_ = MoveEntity {
							is_friction_push: true,
							..MoveEntity::new(id, entity.pos, move_.direction)
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
			sounds.play(SoundId::Push, 0.4);
		}

		// TODO: Resolve move conflicts

		// Run all the moves
		// It's run in reverse because the moves resulting from pushing
		// are always further back in the list, so if we reverse it those
		// are moved first, which allows the pushers to also be moved.
		for move_ in events.moves.iter().rev() {
			let to = move_.to();

			if self.tile_is_solid(to) {
				let entity = self.data.entities.get(&move_.entity_id).unwrap();
				animations.push_back(Animation::FailedMove {
					entity_id: move_.entity_id,
					from: move_.from,
					to,
					accelerate: !entity.is_sliding,
				});
				continue;
			}

			let entity = self.data.entities.get(&move_.entity_id).unwrap();
			if entity.kind == EntityKind::BucketOfGoop {
				match self.data.tiles.get_tile(move_.to()).unwrap() {
					Tile::Ice => {
						self.data.tiles.set_tile(move_.to(), Tile::IceWithGoop);
						self.n_tile_changes += 1;
					}
					Tile::Floor => {
						self.data.tiles.set_tile(move_.to(), Tile::FloorWithGoop);
						self.n_tile_changes += 1;
					}
					Tile::Home | Tile::SadHome => {
						// Cannot move Bucket of Goop onto houses.
						animations.push_back(Animation::FailedMove {
							entity_id: move_.entity_id,
							from: move_.from,
							to,
							accelerate: !entity.is_sliding,
						});
						continue;
					}
					_ => (),
				}
			}

			let entity = self.data.entities.get_mut(&move_.entity_id).unwrap();
			match self.data.tiles.get_tile(to).unwrap() {
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

			entity.pos = to;

			let mut moving_to_ice = false;
			if self.data.tiles.get_tile(to) == Some(Tile::Ice) {
				moving_to_ice = true;
			}

			animations.push_back(Animation::Move {
				entity_id: move_.entity_id,
				from: move_.from,
				to,
				accelerate: !entity.is_sliding,
				decelerate: !moving_to_ice,
				kind: AnimationMoveKind::Standard,
			});
			entity.is_sliding = moving_to_ice;
		}

		// Entities that modify tiles
		let mut entities_to_remove = Vec::new();
		for (&entity_id, entity) in self.data.entities.iter() {
			let mut modified_tile = false;
			match (entity.kind, self.data.tiles.get_tile(entity.pos).unwrap()) {
				(EntityKind::Human, Tile::Home) => {
					self.data.tiles.set_tile(
						entity.pos, 
						Tile::Wall(WallKind::HappyHome),
					);
					self.data.n_humans -= 1;
					if self.data.n_humans == 0 {
						self.has_won = true;
					}
					modified_tile = true;
				}
				(EntityKind::Cake, Tile::SadHome) => {
					self.data.tiles.set_tile(entity.pos, Tile::Home);
					modified_tile = true;
				}
				_ => (),
			}

			if modified_tile {
				self.n_tile_changes += 1;

				let mut from = entity.pos;
				let to = entity.pos;
				let mut accelerate = false;
				for (i, animation) in animations.iter().enumerate() {
					if let Animation::Move { 
						from: anim_from, 
						to: anim_to, 
						accelerate: anim_accelerate, 
						kind: AnimationMoveKind::Standard,
						..
					} = *animation {
						if anim_to == from {
							accelerate = anim_accelerate;
							from = anim_from;
							animations.remove(i);
							break;
						}
					}
				}

				animations.push_back(Animation::Move {
					entity_id,
					from,
					to,
					accelerate,
					decelerate: false,
					kind: AnimationMoveKind::Apply,
				});
				entities_to_remove.push(entity_id);
			}
		}

		for entity in entities_to_remove {
			self.data.entities.remove(&entity);
		}

		if self.data.has_input {
			sounds.play(SoundId::SpiderWalk, 0.3);
			self.data.has_input = false;
		}

		self.data.active_events = new_events;
		self.old_events = Some(events);
	}
}

#[derive(Clone, Default)]
pub struct Tilemap {
	pub width: usize,
	pub height: usize,
	pub buffer: Vec<Tile>,
}

impl Tilemap {
	pub fn get_tile(&self, pos: [isize; 2]) -> Option<Tile> {
		debug_assert_eq!(self.buffer.len(), self.width * self.height);
		if pos[0] < 0 || pos[0] as usize >= self.width 
			|| pos[1] < 0 || pos[1] as usize >= self.height {
			return None;
		}

		// SAFETY: The bounds check is up above
		unsafe {
			Some(*self.buffer.get_unchecked(
				pos[0] as usize + pos[1] as usize * self.width
			))
		}
	}

	/// Sets a tile
	///
	/// # Panics
	/// If the tile is out of bounds.
	pub fn set_tile(&mut self, pos: [isize; 2], tile: Tile) {
		debug_assert_eq!(self.buffer.len(), self.width * self.height);

		if pos[0] < 0 || pos[0] as usize >= self.width 
			|| pos[1] < 0 || pos[1] as usize >= self.height {
			panic!("Tried setting a tile out of bounds!");
		}

		// SAFETY: The bounds check is up above
		unsafe {
			*self.buffer.get_unchecked_mut(
				pos[0] as usize + pos[1] as usize * self.width 
			) = tile;
		}
	}
}

#[derive(Clone, Copy)]
pub enum AnimationMoveKind {
	Standard,
	IceKick,
	Apply,
}

#[derive(Clone, Copy)]
pub enum Animation {
	Move { 
		entity_id: u32, 
		from: [isize; 2], 
		to: [isize; 2],
		accelerate: bool,
		decelerate: bool,
		kind: AnimationMoveKind,
	},
	FailedMove { 
		entity_id: u32, 
		from: [isize; 2], 
		to: [isize; 2], 
		accelerate: bool,
	},
	// TODO: Add particles of goop when something is goopified
	Goopify				{ entity_id: u32, kind: EntityKind },
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
pub enum WallKind {
	Void,
	Grass,
	Flowers,
	HappyHome,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	Floor,
	Wall(WallKind),
	SadHome,
	Home,
	Ice,
	FloorWithGoop,
	IceWithGoop,
}

impl Tile {
	pub fn graphics(self) -> [Option<TileGraphics>; 3] {
		use Tile::*;
		let mut values = [None; 3];

		// The base tilemap.
		values[0] = match self {
			Floor | SadHome | Home | FloorWithGoop => 
				Some(TileGraphics::Tilemap {
					atlas: Texture::FloorMap,
					connects_to_tile: |tile| match tile {
						Wall(_) => false,
						Ice => false,
						IceWithGoop => false,
						_ => true
					},
				}),
			Wall(WallKind::Void) => Some(TileGraphics::Tilemap {
				atlas: Texture::VoidMap,
				connects_to_tile: |tile| match tile {
					Wall(WallKind::Void) => false,
					_ => true,
				}
			}),
			Wall(WallKind::Grass) => unimplemented!(),
			Wall(WallKind::Flowers) => unimplemented!(),
			Wall(WallKind::HappyHome) => None,
			Ice | IceWithGoop =>
				Some(TileGraphics::Tilemap {
					atlas: Texture::IceMap,
					connects_to_tile: |tile| match tile {
						Ice | IceWithGoop => true,
						_ => false,
					},
				})
		};

		// The tile texture
		values[1] = match self {
			Wall(WallKind::HappyHome) => 
				Some(TileGraphics::Texture(Texture::HappyHome)),
			SadHome => Some(TileGraphics::Texture(Texture::SadHome)),
			Home => Some(TileGraphics::Texture(Texture::Home)),
			_ => None,
		};
		
		// Goop layer
		values[2] = match self {
			IceWithGoop | FloorWithGoop => Some(TileGraphics::Tilemap {
				atlas: Texture::GoopMap,
				connects_to_tile: |tile| match tile {
					IceWithGoop | FloorWithGoop => true,
					_ => false
				},
			}),
			_ => None,
		};

		values
	}
}

#[derive(Clone, Copy)]
pub enum TileGraphics {
	Texture(Texture),
	Tilemap {
		atlas: Texture, 
		connects_to_tile: fn(Tile) -> bool,
	},
}

#[derive(Clone, Copy)]
pub struct Entity {
	pub pos: [isize; 2],
	pub kind: EntityKind,
	pub is_sliding: bool,
}

impl Entity {
	pub fn new(x: isize, y: isize, kind: EntityKind) -> Self {
		Entity { pos: [x, y], kind, is_sliding: false, }
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
