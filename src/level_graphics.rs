use crate::prelude::*;
use crate::level::{Tile, Animation, AnimationMoveKind};
use crate::graphics::{TextureVertex, Graphics};
use crate::textures::{UVCoords, Texture as TextureId};
use std::collections::{HashMap, VecDeque};

pub struct LevelGraphics {
	tilemap_change: u32,

	vertices: VertexBuffer<TextureVertex>,
	indices: IndexBuffer<u32>,

	pub animations: VecDeque<Animation>,

	entities: HashMap<u32, EntityGraphics>,

	win_panel_position: f32,
}

impl LevelGraphics {
	pub fn new(graphics: &Graphics, level: &Level) -> LevelGraphics {
		let (vertices, indices) = generate_level_graphics(graphics, level);

		let mut entities = HashMap::new();
		for (id, entity) in level.data.entities.iter() {
			let uv = graphics.textures.get_uv(entity.kind.get_texture());
			let vertices = VertexBuffer::new(&graphics.display,
				&[TextureVertex {
					position: [-0.5, -0.5, 1.0],
					uv: [uv.left, uv.bottom],
				},
				TextureVertex {
					position: [-0.5, 0.5, 1.0],
					uv: [uv.left, uv.top],
				},
				TextureVertex {
					position: [0.5, 0.5, 1.0],
					uv: [uv.right, uv.top],
				},
				TextureVertex {
					position: [0.5, -0.5, 1.0],
					uv: [uv.right, uv.bottom],
				}]
			).unwrap();
			let indices = IndexBuffer::new(&graphics.display,
				index::PrimitiveType::TrianglesList,
				&[0, 1, 2, 0, 2, 3u32],
			).unwrap();

			entities.insert(*id, EntityGraphics {
				position: [entity.x as f32, entity.y as f32],
				size: 1.0,
				vertex_buffer: vertices,
				index_buffer: indices,
			});
		}
	
		LevelGraphics {
			vertices,
			indices,
			entities,
			animations: VecDeque::new(),
			win_panel_position: 0.0,
			tilemap_change: level.n_tile_changes,
		}
	}

	pub fn reset(&mut self, graphics: &Graphics, level: &Level) {
		let (vertices, indices) = generate_level_graphics(graphics, level);
		self.vertices = vertices;
		self.indices = indices;
		self.entities.clear();
		for (id, entity) in level.data.entities.iter() {
			let uv = graphics.textures.get_uv(entity.kind.get_texture());
			let vertices = VertexBuffer::new(&graphics.display,
				&[TextureVertex {
					position: [-0.5, -0.5, 1.0],
					uv: [uv.left, uv.bottom],
				},
				TextureVertex {
					position: [-0.5, 0.5, 1.0],
					uv: [uv.left, uv.top],
				},
				TextureVertex {
					position: [0.5, 0.5, 1.0],
					uv: [uv.right, uv.top],
				},
				TextureVertex {
					position: [0.5, -0.5, 1.0],
					uv: [uv.right, uv.bottom],
				}]
			).unwrap();
			let indices = IndexBuffer::new(&graphics.display,
				index::PrimitiveType::TrianglesList,
				&[0, 1, 2, 0, 2, 3u32],
			).unwrap();

			self.entities.insert(*id, EntityGraphics {
				position: [entity.x as f32, entity.y as f32],
				size: 1.0,
				vertex_buffer: vertices,
				index_buffer: indices,
			});
		}
		self.animations.clear();
		self.tilemap_change = 0;
	}

	pub fn render_level(
		&mut self, 
		graphics: &Graphics,
		surface: &mut impl Surface, 
		aspect: f32,
		level: &mut Level,
		time: f32,
		dt: f32,
	) {
		let size = if (level.data.height as f32) > (level.data.width as f32 / aspect) {
			1.0 / level.data.height as f32
		} else {
			aspect / level.data.width as f32
		};
		let camera_matrix = [
			[1.5 * size / aspect, 0.0, 0.0f32],
			[0.0, 1.5 * size, 0.0f32],
			[0.0, 0.0, 1.0f32],
		];

		let model_transform = [
			[1.0, 0.0, 0.0f32],
			[0.0, 1.0, 0.0f32],
			[-(level.data.width as f32) / 2.0, -(level.data.height as f32) / 2.0, 1.0f32],
		];

		// If the tilemap has changed, change the graphics too!
		if self.tilemap_change < level.n_tile_changes {
			let (vertices, indices) = generate_level_graphics(graphics, level);
			self.vertices = vertices;
			self.indices  = indices;
			self.tilemap_change = level.n_tile_changes;
		}

		// Draw the tilemap
		surface.draw(
			&self.vertices,
			&self.indices,
			&graphics.world_texture_program,
			&uniform! {
				model_transform: model_transform,
				camera_transform: camera_matrix,
				atlas: graphics.textures.atlas.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Nearest),
			},
			&DrawParameters {
				blend: Blend {
					color: BlendingFunction::Addition {
						source: LinearBlendingFactor::One,
						destination: LinearBlendingFactor::OneMinusSourceAlpha,
					},
					..Default::default()
				},
				..Default::default()
			}
		).unwrap();

		// Animate stuff
		for event in self.animations.iter_mut() {
			match *event {
				Animation::Move {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
					accelerate, decelerate,
					kind: AnimationMoveKind::Standard,
				} => {
					let t = smooth_lerp_time(time, accelerate, decelerate);
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
				}
				Animation::Move {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
					accelerate, decelerate,
					kind: AnimationMoveKind::IceKick,
				} => {
					assert!(decelerate);

					const FACTOR: f32 = 2.0;
					let t = 
						double_lerp(
							(time * FACTOR).min(2.0), 
							accelerate
						) / FACTOR;
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
				}
				Animation::Move { 
					entity_id, 
					from: [from_x, from_y], 
					to: [to_x, to_y],
					kind: AnimationMoveKind::Apply,
					accelerate, decelerate,
				} => {
					let t = smooth_lerp_time(time, accelerate, decelerate);
					let mut entity =
						self.entities.get_mut(&entity_id).unwrap();
					entity.position[0] = lerp(from_x as f32, to_x as f32, t);
					entity.position[1] = lerp(from_y as f32, to_y as f32, t);
					entity.size = 1.0 - t;
				}

				Animation::FailedMove {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
					accelerate,
				} => {
					let factor: f32 = if accelerate {
						2.0
					} else {
						4.0
					};
					let t = 
						double_lerp(
							(time * factor).min(2.0), 
							accelerate
						) / 5.0;
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
				}
				Animation::Goopify { entity_id, kind } => {
					let gfx = self.entities.get_mut(&entity_id).unwrap();
					// @Cleanup: Put the entity graphics creation in a function
					let uv = graphics.textures.get_uv(kind.get_texture());
					gfx.vertex_buffer = VertexBuffer::new(&graphics.display,
						&[TextureVertex {
							position: [-0.5, -0.5, 1.0],
							uv: [uv.left, uv.bottom],
						},
						TextureVertex {
							position: [-0.5, 0.5, 1.0],
							uv: [uv.left, uv.top],
						},
						TextureVertex {
							position: [0.5, 0.5, 1.0],
							uv: [uv.right, uv.top],
						},
						TextureVertex {
							position: [0.5, -0.5, 1.0],
							uv: [uv.right, uv.bottom],
						}]
					).unwrap();
					gfx.index_buffer = IndexBuffer::new(&graphics.display,
						index::PrimitiveType::TrianglesList,
						&[0, 1, 2, 0, 2, 3u32],
					).unwrap();
				}
			}
		}

		for entity_graphics in self.entities.values() {
			surface.draw(
				&entity_graphics.vertex_buffer,
				&entity_graphics.index_buffer,
				&graphics.world_texture_program,
				&uniform! {
					model_transform: crate::matrix::matrix_mul(model_transform, [
						[entity_graphics.size, 0.0, 0.0],
						[0.0, entity_graphics.size, 0.0],
						[entity_graphics.position[0] + 0.5, entity_graphics.position[1] + 0.5, 1.0],
					]),
					camera_transform: camera_matrix,
					atlas: graphics.textures.atlas.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Nearest),
				},
				&DrawParameters {
					blend: Blend {
						color: BlendingFunction::Addition {
							source: LinearBlendingFactor::One,
							destination: LinearBlendingFactor::OneMinusSourceAlpha,
						},
						..Default::default()
					},
					..Default::default()
				}
			).unwrap();
		}

		if level.has_won {
			self.win_panel_position = 
				1.0f32.min(self.win_panel_position + dt * 4.0);
		}else {
			self.win_panel_position = 
				0.0f32.max(self.win_panel_position - dt * 4.0);
		};

		if self.win_panel_position > 0.05 {
			let t = 1.0 - self.win_panel_position;
			let t = 1.0 - (t * t);

			let panel_y = lerp(-2.0, 0.0, t);
			graphics.draw_texture_immediate(
				surface, 
				aspect, 
				[-1.0, panel_y - 0.25, 1.0, panel_y + 0.25], 
				TextureId::VictoryText,
			);
		}
	}
}

struct EntityGraphics {
	position: [f32; 2],
	size: f32,
	vertex_buffer: VertexBuffer<TextureVertex>,
	index_buffer: IndexBuffer<u32>,
}

fn generate_level_graphics(
	graphics: &Graphics,
	level: &Level, 
) -> (VertexBuffer<TextureVertex>, IndexBuffer<u32>) {
	let mut vertices = Vec::new();
	let mut indices = Vec::new();

	for y in -1..=level.data.height as isize {
		for x in -1..=level.data.width as isize {
			let tile = level.get_tile([x, y]).unwrap_or(Tile::Wall);
			
			if tile == Tile::Wall {
				let mut data = [false; 9];
				for rel_x in 0..=2 {
					for rel_y in 0..=2 {
						data[rel_x as usize + rel_y as usize * 3] = 
							level.get_tile([x + rel_x - 1, y + rel_y - 1])
							.map(|v| v != Tile::Wall)
							.unwrap_or(false);
					}
				}
				let uv = graphics.textures.get_uv(TextureId::VoidMap);
				generate_tilemap_tile_graphics(
					graphics,
					[x as f32, y as f32, 1.0, 1.0],
					uv,
					data,
					&mut vertices,
					&mut indices,
				);
				continue;
			}

			if !(tile == Tile::Wall || tile == Tile::Ice || tile == Tile::IceWithGoop) {
				let mut data = [false; 9];
				for rel_x in 0..=2 {
					for rel_y in 0..=2 {
						data[rel_x as usize + rel_y as usize * 3] = 
							level.get_tile([x + rel_x - 1, y + rel_y - 1])
							.map(|v| v != Tile::HappyHome && v != Tile::Wall && v != Tile::Ice && v != Tile::IceWithGoop)
							.unwrap_or(false);
					}
				}

				let uv = graphics.textures.get_uv(TextureId::FloorMap);
				generate_tilemap_tile_graphics(
					graphics,
					[x as f32, y as f32, 1.0, 1.0],
					uv,
					data,
					&mut vertices,
					&mut indices,
				);

				let uv = match tile {
					Tile::Floor => None,
					Tile::Ice => unreachable!(),
					Tile::Home => Some(TextureId::Home),
					Tile::Wall => None,
					Tile::HappyHome => Some(TextureId::HappyHome),
					Tile::SadHome => Some(TextureId::SadHome),
					Tile::FloorWithGoop => None,
					Tile::IceWithGoop => unreachable!(),
				}.map(|v| graphics.textures.get_uv(v));

				if let Some(uv) = uv {
					graphics.push_texture_quad(
						&mut vertices,
						&mut indices,
						[x as f32, y as f32, 1.0, 1.0],
						uv
					);
				}
				continue;
			} else if tile == Tile::Ice || tile == Tile::IceWithGoop {
				let mut data = [false; 9];
				for rel_x in 0..=2 {
					for rel_y in 0..=2 {
						data[rel_x as usize + rel_y as usize * 3] = 
							level.get_tile([x + rel_x - 1, y + rel_y - 1])
							.map(|v| v == Tile::Ice || v == Tile::IceWithGoop)
							.unwrap_or(false);
					}
				}

				let uv = graphics.textures.get_uv(TextureId::IceMap);
				generate_tilemap_tile_graphics(
					graphics,
					[x as f32, y as f32, 1.0, 1.0],
					uv,
					data,
					&mut vertices,
					&mut indices,
				);
			}
		}
	}

	for (i, tile) in level.data.tiles.iter().copied().enumerate() {
		let x = (i % level.data.width) as isize;
		let y = (i / level.data.width) as isize;
		if tile == Tile::IceWithGoop || tile == Tile::FloorWithGoop {
			let mut data = [false; 9];
			for rel_x in 0..=2 {
				for rel_y in 0..=2 {
					data[rel_x as usize + rel_y as usize * 3] = 
						level.get_tile([x + rel_x - 1, y + rel_y - 1])
						.map(|v| v == Tile::IceWithGoop || v == Tile::FloorWithGoop)
						.unwrap_or(false);
				}
			}

			let uv = graphics.textures.get_uv(TextureId::GoopMap);
			generate_tilemap_tile_graphics(
				graphics,
				[x as f32, y as f32, 1.0, 1.0],
				uv,
				data,
				&mut vertices,
				&mut indices,
			);
		}
	}

	(
		VertexBuffer::new(&graphics.display, &vertices).unwrap(),
		IndexBuffer::new(
			&graphics.display, 
			glium::index::PrimitiveType::TrianglesList, 
			&indices,
		).unwrap(),
	)
}

/// A function where 0 <= t <= 2, 0 and 2 will give 0.
#[inline]
fn double_lerp(t: f32, accelerate: bool) -> f32 {
	if t < 1.0 {
		smooth_lerp_time(t, accelerate, true)
	}else {
		1.0 - smooth_lerp_time(t - 1.0, true, true)
	}
}

#[inline]
fn smooth_lerp_time(t: f32, accelerate: bool, decelerate: bool) -> f32 {
	match (accelerate, decelerate) {
		(false, false) => t,
		(true, false)  => -t * t * t + 2.0 * t * t,
		(false, true)  => 1.0 - smooth_lerp_time(1.0 - t, true, false),
		(true, true)   => 3.0 * t * t - 2.0 * t * t * t,
	}
}

/// Pass a graphics, a position rectangle, atlas coordinates and the surrounding
/// info, and generate a valid thing!
///
/// The surrounding info is a map like this:
/// 6 7 9
/// 3 4 5
/// 0 1 2
/// where 4 is the tile being rendered(ignored),
/// and the other ones are true if they are "set".
/// What it means for a tile to be "set" is up to the caller.
fn generate_tilemap_tile_graphics(
	graphics: &Graphics,
	pos: [f32; 4],
	atlas_coords: UVCoords,
	data: [bool; 9],
	vertices: &mut Vec<TextureVertex>,
	indices: &mut Vec<u32>,
) {
	let mut add_quad = move |
		uv: UVCoords,
		pos: [f32; 4],
		horizontal: bool,
		vertical: bool,
		diagonal: bool,
	| {
		let offset = 2.0 *
			(if horizontal { 4.0 } else { 0.0 } + 
			 if vertical   { 2.0 } else { 0.0 } +
			 if diagonal   { 1.0 } else { 0.0 });
		graphics.push_texture_quad(
			vertices,
			indices,
			pos,
			uv.relative(offset, 0.0, offset + 1.0, 1.0),
		);
	};

	let atlas_coords = atlas_coords.relative(0.0, 0.0, 1.0 / 8.0, 1.0);
	// Bottom left
	add_quad(
		atlas_coords.relative(0.0, 0.0, 0.5, 0.5),
		[pos[0], pos[1], pos[2] / 2.0, pos[3] / 2.0],
		data[3],
		data[1],
		data[0],
	);
	// Bottom right
	add_quad(
		atlas_coords.relative(0.5, 0.0, 1.0, 0.5),
		[pos[0] + pos[2] / 2.0, pos[1], pos[2] / 2.0, pos[3] / 2.0],
		data[5],
		data[1],
		data[2],
	);
	// Top right
	add_quad(
		atlas_coords.relative(0.5, 0.5, 1.0, 1.0),
		[pos[0] + pos[2] / 2.0, pos[1] + pos[3] / 2.0, pos[2] / 2.0, pos[3] / 2.0],
		data[5],
		data[7],
		data[8],
	);
	// Top left
	add_quad(
		atlas_coords.relative(0.0, 0.5, 0.5, 1.0),
		[pos[0], pos[1] + pos[3] / 2.0, pos[2] / 2.0, pos[3] / 2.0],
		data[3],
		data[7],
		data[6],
	);
}
