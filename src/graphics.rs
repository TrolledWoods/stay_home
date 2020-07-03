use crate::prelude::*;
use crate::textures::{Textures, UVCoords};
use crate::sounds::Sounds;

pub struct Graphics {
	pub world_texture_program: Program,
	pub background_program: Program,
	pub textures: Textures,
	pub sounds: Sounds,
	pub display: Display,
}

impl Graphics {
	pub fn new(display: &Display, sounds: Sounds) -> Self {
		println!("Compiling texture shader...");
		let world_texture_program = Program::from_source(display, TEXTURE_VERTEX_SHADER, TEXTURE_FRAGMENT_SHADER, None).unwrap();
		println!("Compiling background shader...");
		let background_program = Program::from_source(display, BACKGROUND_VERTEX_SHADER, BACKGROUND_FRAGMENT_SHADER, None).unwrap();

		// @Cleanup: Don't unwrap here, silly!
		let textures = Textures::load("assets/", &display).unwrap();

		Graphics {
			sounds,
			world_texture_program,
			background_program,
			textures,
			display: display.clone(),
		}
	}

	// pub fn draw_texture_immediate(&self, surface: &mut impl Surface, aspect: f32, rect: [f32; 4], texture: TextureId) {
	// 	let uv = self.textures.get_uv(texture);
	// 	let vertices = VertexBuffer::new(&self.display,
	// 		&[TextureVertex {
	// 			position: [rect[0], rect[1], 1.0],
	// 			uv: [uv.left, uv.bottom],
	// 		},
	// 		TextureVertex {
	// 			position: [rect[0], rect[3], 1.0],
	// 			uv: [uv.left, uv.top],
	// 		},
	// 		TextureVertex {
	// 			position: [rect[2], rect[3], 1.0],
	// 			uv: [uv.right, uv.top],
	// 		},
	// 		TextureVertex {
	// 			position: [rect[2], rect[1], 1.0],
	// 			uv: [uv.right, uv.bottom],
	// 		}]
	// 	).unwrap();
	// 	let indices = IndexBuffer::new(&self.display,
	// 		index::PrimitiveType::TrianglesList,
	// 		&[0, 1, 2, 0, 2, 3u32],
	// 	).unwrap();

	// 	surface.draw(
	// 		&vertices,
	// 		&indices,
	// 		&self.world_texture_program,
	// 		&uniform! {
	// 			model_transform: [
	// 				[1.0, 0.0, 0.0f32],
	// 				[0.0, 1.0, 0.0f32],
	// 				[0.0, 0.0, 1.0f32],
	// 			],
	// 			camera_transform: [
	// 				[1.0 / aspect, 0.0, 0.0f32],
	// 				[0.0, 1.0, 0.0f32],
	// 				[0.0, 0.0, 1.0f32],
	// 			],
	// 			atlas: self.textures.atlas.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Nearest),
	// 		},
	// 		&DrawParameters {
	// 			blend: Blend {
	// 				color: BlendingFunction::Addition {
	// 					source: LinearBlendingFactor::One,
	// 					destination: LinearBlendingFactor::OneMinusSourceAlpha,
	// 				},
	// 				..Default::default()
	// 			},
	// 			..Default::default()
	// 		}
	// 	).unwrap();
	// }
	
	pub fn draw_background_immediate(&self, surface: &mut impl Surface, aspect: f32, rect: [f32; 4], uv: UVCoords, time: f32) {
		let vertices = VertexBuffer::new(&self.display,
			&[BackgroundVertex {
				position: [rect[0], rect[1], 1.0],
				uv: [uv.left, uv.bottom],
			},
			BackgroundVertex {
				position: [rect[0], rect[3], 1.0],
				uv: [uv.left, uv.top],
			},
			BackgroundVertex {
				position: [rect[2], rect[3], 1.0],
				uv: [uv.right, uv.top],
			},
			BackgroundVertex {
				position: [rect[2], rect[1], 1.0],
				uv: [uv.right, uv.bottom],
			}]
		).unwrap();
		let indices = IndexBuffer::new(&self.display,
			index::PrimitiveType::TrianglesList,
			&[0, 1, 2, 0, 2, 3u32],
		).unwrap();

		surface.draw(
			&vertices,
			&indices,
			&self.background_program,
			&uniform! {
				time: time,
			},
			&DrawParameters {
				..Default::default()
			}
		).unwrap();
	}

	pub fn push_texture_quad(&self, 
		vertices: &mut Vec<TextureVertex>,
		indices: &mut Vec<u32>,
		pos: [f32; 4], 
		uv: UVCoords,
	) {
		let vert_index = vertices.len() as u32;
		vertices.push(TextureVertex {
			position: [pos[0] as f32, pos[1] as f32, 1.0],
			uv: [uv.left, uv.bottom],
		});
		vertices.push(TextureVertex {
			position: [pos[0] as f32, pos[1] as f32 + pos[3], 1.0],
			uv: [uv.left, uv.top],
		});
		vertices.push(TextureVertex {
			position: [pos[0] as f32 + pos[2], pos[1] as f32 + pos[3], 1.0],
			uv: [uv.right, uv.top],
		});
		vertices.push(TextureVertex {
			position: [pos[0] as f32 + pos[2], pos[1] as f32, 1.0],
			uv: [uv.right, uv.bottom],
		});

		indices.push(vert_index);
		indices.push(vert_index + 1);
		indices.push(vert_index + 2);

		indices.push(vert_index);
		indices.push(vert_index + 2);
		indices.push(vert_index + 3);
	}
}

#[derive(Clone, Copy)]
pub struct BackgroundVertex {
	pub position: [f32; 3],
	pub uv: [f32; 2],
}

implement_vertex!(BackgroundVertex, position, uv);

const BACKGROUND_VERTEX_SHADER: &str = r##"
#version 130

uniform float time;

in vec3 position;
in vec2 uv;
out vec2 out_uv;
out float out_time;

void main() {
	out_uv = uv;
	out_time = time;
	gl_Position = vec4(position, 1.0);
}
"##;

const BACKGROUND_FRAGMENT_SHADER: &str = r##"
#version 130

in float out_time;
in vec2 out_uv;

void main() {
	float x = floor(out_uv.x);
	float y = floor(out_uv.y);
	float real_c = (x + y + sin(out_time * 0.1)) / 32.0;
	
	float colors = 2.0;
	float floor_c = floor(real_c * colors) / colors;
	float error_c = floor((real_c - floor_c) * 50.0) / 50.0;
	float c = floor_c + floor(mod(x * 13.0 - y * 11.0, 1.0 + error_c) * colors) / colors;
	gl_FragColor = vec4(c / 70.0 + 0.2, c / 70.0 + 0.2, c / 60.0 + 0.3, 1.0);
}
"##;

#[derive(Clone, Copy)]
pub struct TextureVertex {
	pub position: [f32; 3],
	pub uv: [f32; 2],
}

implement_vertex!(TextureVertex, position, uv);

const TEXTURE_VERTEX_SHADER: &str = r##"
#version 130

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

const TEXTURE_FRAGMENT_SHADER: &str = r##"
#version 130

uniform sampler2D atlas;

in vec2 out_uv;

void main() {
	gl_FragColor = texture(atlas, out_uv);
}
"##;
