use crate::prelude::*;
use crate::level::{Tile, Animation};
use crate::graphics::{TextureVertex, Graphics};
use crate::textures::Texture as TextureId;
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
		for (id, entity) in level.entities.iter() {
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
		for (id, entity) in level.entities.iter() {
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
		let camera_matrix = [
			[1.0 / (level.height as f32 * aspect), 0.0, 0.0f32],
			[0.0, 1.0 / (level.height as f32), 0.0f32],
			[0.0, 0.0, 1.0f32],
		];

		let model_transform = [
			[1.0, 0.0, 0.0f32],
			[0.0, 1.0, 0.0f32],
			[-(level.width as f32) / 2.0, -(level.height as f32) / 2.0, 1.0f32],
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
			&Default::default(),
		).unwrap();

		// Animate stuff
		for event in self.animations.iter_mut() {
			match *event {
				Animation::Move {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
				} => {
					let t = (time *  time) * (3.0 - 2.0 * time);
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
				}
				Animation::TileModification {
					entity_id,
					..
				} => {
					let t = (time * time) * (3.0 - 2.0 * time);

					// @Cleanup: Don't unwrap here, dummy!
					let mut entity =
						self.entities.get_mut(&entity_id).unwrap();
					entity.size = 1.0 - t;
				}
				Animation::FailedMove {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
				} => {
					let mut t = (time * time) * (3.0 - 2.0 * time);
					t = (0.5 - (t - 0.5).abs()) * 0.2;
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
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

	for (i, tile) in level.tiles.iter().copied().enumerate() {
		let x = i % level.width;
		let y = i / level.width;

		let uv = graphics.textures.get_uv(match tile {
			Tile::Floor => TextureId::Floor,
			Tile::Home => TextureId::Home,
			Tile::Wall => TextureId::Wall,
			Tile::HappyHome => TextureId::HappyHome,
			Tile::SadHome => TextureId::SadHome,
			Tile::Ice => TextureId::Ice,
		});
		let vert_index = vertices.len() as u32;
		vertices.push(TextureVertex {
			position: [x as f32, y as f32, 1.0],
			uv: [uv.left, uv.bottom],
		});
		vertices.push(TextureVertex {
			position: [x as f32, y as f32 + 1.0, 1.0],
			uv: [uv.left, uv.top],
		});
		vertices.push(TextureVertex {
			position: [x as f32 + 1.0, y as f32 + 1.0, 1.0],
			uv: [uv.right, uv.top],
		});
		vertices.push(TextureVertex {
			position: [x as f32 + 1.0, y as f32, 1.0],
			uv: [uv.right, uv.bottom],
		});

		indices.push(vert_index);
		indices.push(vert_index + 1);
		indices.push(vert_index + 2);

		indices.push(vert_index);
		indices.push(vert_index + 2);
		indices.push(vert_index + 3);
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
