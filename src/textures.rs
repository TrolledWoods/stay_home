use crate::prelude::*;
use std::path::Path;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Texture {
	Wall = 0,
	Floor = 1,
	Home = 2,
	Human = 3,
	VictoryText = 4,
	HappyHome = 5,
	Cake = 6,
	SadHome = 7,
	Player = 8,
	Ice = 9,
	HumanWithGoop = 10,
	CakeWithGoop = 11,
	FloorWithGoop = 12,
	IceWithGoop = 13,
	BucketOfGoop = 14,
	FloorMap = 15,
}

pub struct Textures {
	pub atlas: Texture2d,
	coords: Vec<UVCoords>,
} 

impl Textures {
	pub fn load(folder: impl AsRef<Path>, display: &Display) -> Result<Textures, String> {
		use glium::texture::RawImage2d;

		let mut total_width = 0;
		let mut total_height = 0;

		const IMAGE_PATHS: &[&str] = &[
			"wall.png",
			"floor.png",
			"home.png",
			"human.png",
			"victory_text.png",
			"happy_home.png",
			"cake.png",
			"sad_home.png",
			"player.png",
			"ice.png",
			"human_with_goop.png",
			"cake_with_goop.png",
			"floor_with_goop.png",
			"ice_with_goop.png",
			"bucket_of_goop.png",
			"floor_map.png",
		];

		let mut images = Vec::with_capacity(IMAGE_PATHS.len());
		for image_path in IMAGE_PATHS {
			println!("Loading texture '{}'", image_path);
			let image = image::open(&folder.as_ref().join(image_path))
				.map_err(|_| 
					format!("Image '{}' does not exist or is invalid", image_path)
				)?.into_rgba();
			let (width, height) = image.dimensions();
			let image = 
				RawImage2d::from_raw_rgba(image.into_raw(), (width, height));
			total_width += width;
			total_height = total_height.max(height);
			images.push(image);
		}

		println!("Packing textures into atlas...");
		let atlas = Texture2d::empty(display, total_width, total_height)
			.unwrap();
		let mut coords = Vec::with_capacity(images.len());
		let mut x = 0;
		for image in images {
			let rect = Rect {
				left: x,
				bottom: 0,
				width: image.width,
				height: image.height,
			};
			coords.push(UVCoords {
				left: x as f32 / total_width as f32,
				top: 0.0,
				right: (x + image.width) as f32 / total_width as f32,
				bottom: image.height as f32 / total_height as f32,
			});
			x += image.width;

			atlas.write(rect, image);
		}

		Ok(Textures {
			atlas,
			coords,
		})
	}

	pub fn get_uv(&self, texture: Texture) -> UVCoords {
		self.coords[texture as usize]
	}
}

#[derive(Clone, Copy, Debug)]
pub struct UVCoords {
	pub left: f32,
	pub right: f32,
	pub top: f32,
	pub bottom: f32,
}

impl UVCoords {
	pub fn relative(self, left: f32, bottom: f32, right: f32, top: f32) 
		-> UVCoords 
	{
		UVCoords {
			left: lerp(self.left, self.right, left),
			bottom: lerp(self.bottom, self.top, bottom),
			right: lerp(self.left, self.right, right),
			top: lerp(self.bottom, self.top, top),
		}
	}
}
