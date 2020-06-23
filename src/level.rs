use crate::prelude::*;
use crate::graphics::LevelGraphics;
use std::collections::HashMap;

pub struct Level {
	pub n_humans: usize,

	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,

	entity_id_ctr: u32,
	pub entities: HashMap<u32, Entity>,

	pub graphics: Option<LevelGraphics>,
}

impl Level {
	pub fn from_string(input: &str) -> Result<Level, String> {
		let mut n_humans = 0;
		let mut tiles = Vec::new();
		let mut entities = HashMap::new();
		let mut entity_id_ctr = 0;
		let mut width = 0;
		let mut height = 0;

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

		Ok(Level {
			n_humans,
			width,
			height,
			tiles,
			entity_id_ctr,
			entities,
			graphics: None,
		})
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
	pub graphics: Option<()>,
}

impl Entity {
	pub fn new(x: usize, y: usize, kind: EntityKind) -> Self {
		Entity { x, y, kind, graphics: None }
	}
}

#[derive(Clone, Copy, Debug)]
pub enum EntityKind {
	Human,
	Block,
	Cake,
}
