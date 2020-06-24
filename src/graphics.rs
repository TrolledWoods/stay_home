use crate::prelude::*;
use crate::level::Tile;
use crate::textures::{Textures, Texture as TextureId};

pub struct Graphics {
	pub world_texture_program: Program,
	pub textures: Textures,
	pub display: Display,
}

impl Graphics {
	pub fn new(display: &Display) -> Self {
		println!("Compiling shaders...");
		let world_texture_program = Program::from_source(display, TEXTURE_VERTEX_SHADER, TEXTURE_FRAGMENT_SHADER, None).unwrap();

		println!("Loading textures...");
		let textures = Textures::load("assets/", &display).unwrap();

		Graphics {
			world_texture_program,
			textures,
			display: display.clone(),
		}
	}

	pub fn draw_texture_immediate(&self, surface: &mut impl Surface, aspect: f32, rect: [f32; 4], texture: TextureId) {
		let uv = self.textures.get_uv(texture);
		let vertices = VertexBuffer::new(&self.display,
			&[TextureVertex {
				position: [rect[0], rect[1], 1.0],
				uv: [uv.left, uv.bottom],
			},
			TextureVertex {
				position: [rect[0], rect[3], 1.0],
				uv: [uv.left, uv.top],
			},
			TextureVertex {
				position: [rect[2], rect[3], 1.0],
				uv: [uv.right, uv.top],
			},
			TextureVertex {
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
			&self.world_texture_program,
			&uniform! {
				model_transform: [
					[1.0, 0.0, 0.0f32],
					[0.0, 1.0, 0.0f32],
					[0.0, 0.0, 1.0f32],
				],
				camera_transform: [
					[1.0 / aspect, 0.0, 0.0f32],
					[0.0, 1.0, 0.0f32],
					[0.0, 0.0, 1.0f32],
				],
				atlas: self.textures.atlas.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Nearest),
			},
			&Default::default(),
		).unwrap();
	}

}

#[derive(Clone, Copy)]
pub struct TextureVertex {
	pub position: [f32; 3],
	pub uv: [f32; 2],
}

implement_vertex!(TextureVertex, position, uv);

const TEXTURE_VERTEX_SHADER: &str = r##"
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

const TEXTURE_FRAGMENT_SHADER: &str = r##"
#version 150

uniform sampler2D atlas;

in vec2 out_uv;

void main() {
	gl_FragColor = texture(atlas, out_uv);
}
"##;
