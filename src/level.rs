use crate::prelude::*;
use crate::textures::Texture;
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
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
		let entity = self.entities.get(&self.player_id).unwrap();
		self.active_events.moves.push(MoveEntity {
			entity_id: self.player_id,
			from: [entity.x, entity.y],
			direction: input,
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

	pub fn update(&mut self, animations: &mut VecDeque<Animation>) {
		let mut events = std::mem::replace(
			&mut self.active_events, 
			self.old_events.take().unwrap_or_else(|| Events::new()),
		);

		// TODO: Pushing logic

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
			entity.x = to[0];
			entity.y = to[1];

			animations.push_back(Animation::Move {
				entity_id: move_.entity_id,
				from: move_.from,
				to,
			});
		}

		self.old_events = Some(events);
	}
}

#[derive(Clone, Copy)]
pub enum Animation {
	Move { entity_id: u32, from: [isize; 2], to: [isize; 2] },
	FailedMove { entity_id: u32, from: [isize; 2], to: [isize; 2] },
	TileModification { entity_id: u32, from: [isize; 2], at: [isize; 2], new_tile: Tile },
}

#[derive(Clone)]
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
	entity_id: u32,
	from: [isize; 2],
	direction: Input,
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
