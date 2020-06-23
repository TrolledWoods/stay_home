use crate::prelude::*;
use crate::level::Tile;
use crate::graphics::{TextureVertex, Graphics};
use crate::textures::Texture as TextureId;
use std::collections::HashMap;

pub struct LevelGraphics {
	// camera_matrix: [f32; 9],

	vertices: VertexBuffer<TextureVertex>,
	indices: IndexBuffer<u32>,

	entities: HashMap<u32, EntityGraphics>,
}

impl LevelGraphics {
	pub fn new(graphics: &Graphics, level: &Level) -> LevelGraphics {
		let (vertices, indices) = generate_level_graphics(graphics, level);

		let mut entities = HashMap::new();
		for (id, entity) in level.entities.iter() {
			let uv = graphics.textures.get_uv(TextureId::Human);
			let vertices = VertexBuffer::new(&graphics.display,
				&[TextureVertex {
					position: [0.0, 0.0, 1.0],
					uv: [uv.left, uv.bottom],
				},
				TextureVertex {
					position: [0.0, 1.0, 1.0],
					uv: [uv.left, uv.top],
				},
				TextureVertex {
					position: [1.0, 1.0, 1.0],
					uv: [uv.right, uv.top],
				},
				TextureVertex {
					position: [1.0, 0.0, 1.0],
					uv: [uv.right, uv.bottom],
				}]
			).unwrap();
			let indices = IndexBuffer::new(&graphics.display,
				index::PrimitiveType::TrianglesList,
				&[0, 1, 2, 0, 2, 3u32],
			).unwrap();

			entities.insert(*id, EntityGraphics {
				vertex_buffer: vertices,
				index_buffer: indices,
			});
		}
	
		LevelGraphics {
			vertices,
			indices,
			entities,
		}
	}

	pub fn render_level(
		&mut self, 
		graphics: &Graphics,
		surface: &mut impl Surface, 
		aspect: f32,
		level: &mut Level,
		time: f32,
	) {
		let camera_matrix = [
			[1.0 / (level.height as f32 * aspect), 0.0, 0.0f32],
			[0.0, 1.0 / (level.height as f32), 0.0f32],
			[0.0, 0.0, 1.0f32],
		];

		let model_transform = [
			[1.0, 0.0, 0.0f32],
			[0.0, 1.0, 0.0f32],
			[-(level.width as f32) / 2.0, 4.0 * (time / 30.0).sin() -(level.height as f32) / 2.0, 1.0f32],
		];

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

		for (&id, entity_graphics) in self.entities.iter() {
			let entity_data = match level.entities.get(&id) {
				Some(entity_data) => entity_data,
				None => continue,
			};
			surface.draw(
				&entity_graphics.vertex_buffer,
				&entity_graphics.index_buffer,
				&graphics.world_texture_program,
				&uniform! {
					model_transform: crate::matrix::matrix_mul(model_transform, [
						[1.0, 0.0, 0.0],
						[0.0, 1.0, 0.0],
						[entity_data.x as f32, entity_data.y as f32, 1.0],
					]),
					camera_transform: camera_matrix,
					atlas: graphics.textures.atlas.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Nearest),
				},
				&Default::default(),
			).unwrap();
		}
	}
}

struct EntityGraphics {
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
