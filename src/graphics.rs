use crate::prelude::*;
use crate::level::Tile;

pub struct Graphics {
	level_program: Program,
}

impl Graphics {
	pub fn new(display: &Display) -> Graphics {
		Graphics {
			level_program: Program::from_source(display, LEVEL_VERTEX_SHADER, LEVEL_FRAGMENT_SHADER, None).unwrap(),
		}
	}

	pub fn render_level(
		&mut self, 
		display: &Display, 
		surface: &mut impl Surface, 
		aspect: f32,
		level: &mut Level,
	) {
		let mut graphics = level.graphics.take().unwrap_or_else(|| {
			generate_level_graphics(display, level)
		});

		surface.draw(
			&graphics.vertices,
			&graphics.indices,
			&self.level_program,
			&uniform! {
				model_transform: [
					[1.0,   0.0, 0.0f32],
					[0.0,   1.0, 0.0f32],
					[-1.5, -1.5, 1.0f32],
				],
				camera_transform: [
					[0.4 / aspect, 0.0, 0.0f32],
					[0.0, 0.4, 0.0f32],
					[0.0, 0.0, 1.0f32],
				],
			},
			&Default::default(),
		).unwrap();

		level.graphics = Some(graphics);
	}
}

pub struct LevelGraphics {
	vertices: VertexBuffer<LevelVertex>,
	indices: IndexBuffer<u32>,
}

fn generate_level_graphics(display: &Display, level: &Level) -> LevelGraphics {
	let mut vertices = Vec::new();
	let mut indices = Vec::new();

	for (i, tile) in level.tiles.iter().copied().enumerate() {
		let x = i % level.width;
		let y = i / level.width;

		let vert_index = vertices.len() as u32;
		vertices.push(LevelVertex {
			position: [x as f32, y as f32, 1.0],
			uv: [tile.id() as f32 / Tile::n_tiles() as f32, 0.0f32],
		});
		vertices.push(LevelVertex {
			position: [x as f32, y as f32 + 1.0, 1.0],
			uv: [tile.id() as f32 / Tile::n_tiles() as f32, 0.0f32],
		});
		vertices.push(LevelVertex {
			position: [x as f32 + 1.0, y as f32 + 1.0, 1.0],
			uv: [(tile.id() as f32 + 0.0) / Tile::n_tiles() as f32, 0.0f32],
		});
		vertices.push(LevelVertex {
			position: [x as f32 + 1.0, y as f32, 1.0],
			uv: [(tile.id() as f32 + 0.0) / Tile::n_tiles() as f32, 0.0f32],
		});

		indices.push(vert_index);
		indices.push(vert_index + 1);
		indices.push(vert_index + 2);

		indices.push(vert_index);
		indices.push(vert_index + 2);
		indices.push(vert_index + 3);
	}

	LevelGraphics {
		vertices: VertexBuffer::new(display, &vertices).unwrap(),
		indices: IndexBuffer::new(
			display, 
			glium::index::PrimitiveType::TrianglesList, 
			&indices,
		).unwrap(),
	}
}

#[derive(Clone, Copy)]
pub struct LevelVertex {
	pub position: [f32; 3],
	pub uv: [f32; 2],
}

implement_vertex!(LevelVertex, position, uv);

const LEVEL_VERTEX_SHADER: &str = r##"
#version 150

uniform mat3 model_transform;
uniform mat3 camera_transform;

in vec3 position;
in vec2 uv;
out vec2 out_uv;

void main() {
	out_uv = uv;
	gl_Position = vec4(camera_transform * model_transform * position, 1.0);
}
"##;

const LEVEL_FRAGMENT_SHADER: &str = r##"
#version 150

in vec2 out_uv;

void main() {
	gl_FragColor = vec4(out_uv, 1.0, 1.0);
}
"##;
