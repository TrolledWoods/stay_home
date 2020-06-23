use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Level {
	pub n_humans: usize,

	pub width: usize,
	pub height: usize,
	pub tiles: Vec<Tile>,
	// pub entities: Vec<Entity>,
}

impl Level {
	pub fn from_string(input: &str) -> Result<Level, String> {
		let mut n_humans = 0;
		let mut tiles = Vec::new();
		let mut width = 0;
		let mut height = 0;
		for line in input.lines()
			.map(|v| v.trim())
			.filter(|v| !v.is_empty()) 
		{
			if width == 0 { 
				width = line.len(); 
			}else if width != line.len() { 
				return Err(format!("Expected the same width for every line")); 
			}

			height += 1;

			for (char_index, char_) in line.chars().enumerate() {
				tiles.push(match char_ {
					// Entities
					'@' => todo!("Human"),
					'B' => todo!("Block"),
					'C' => todo!("Cake"),

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
		})
	}
}

// #[derive(Clone, Copy)]
// pub enum EntityKind {
// 	Human,
// 	Block,
// 	Cake,
// }

#[derive(Clone, Copy, Debug)]
pub enum Tile {
	Home,
	Floor,
	Wall,
}
