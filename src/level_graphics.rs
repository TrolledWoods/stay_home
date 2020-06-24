use crate::prelude::*;
use crate::level::Tile;
use crate::graphics::{TextureVertex, Graphics};
use crate::textures::Texture as TextureId;
use std::collections::{HashMap, VecDeque};

pub struct LevelGraphics {
	// camera_matrix: [f32; 9],

	vertices: VertexBuffer<TextureVertex>,
	indices: IndexBuffer<u32>,

	pub animations: VecDeque<(f32, Event)>,

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
				position: [entity.x as f32, entity.y as f32],
				vertex_buffer: vertices,
				index_buffer: indices,
			});
		}
	
		LevelGraphics {
			vertices,
			indices,
			entities,
			animations: VecDeque::new(),
		}
	}

	pub fn render_level(
		&mut self, 
		graphics: &Graphics,
		surface: &mut impl Surface, 
		aspect: f32,
		level: &mut Level,
		delta_time: f32,
	) {
		// Animate stuff
		let n_animations = self.animations.len();
		for &mut (ref mut timer, event) in self.animations.iter_mut() {
			*timer = 1.0f32.min(*timer + delta_time * 7.0);

			match event {
				Event::EntityMoved {
					entity_id,
					from: [from_x, from_y],
					to: [to_x, to_y],
				} => {
					let t = (*timer *  *timer) * (3.0 - 2.0 * *timer);
					let lerp_x = lerp(from_x as f32, to_x as f32, t);
					let lerp_y = lerp(from_y as f32, to_y as f32, t);

					// @Cleanup: Don't unwrap here, dummy!
					self.entities.get_mut(&entity_id).unwrap().position 
						= [lerp_x, lerp_y];
				}
				// Unanimated event
				_ => ()
			}
		}

		self.animations.retain(|&(t, _)| t < 1.0);

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
						[entity_graphics.position[0], entity_graphics.position[1], 1.0],
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
	}
}

struct EntityGraphics {
	position: [f32; 2],
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
