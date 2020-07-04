use crate::prelude::*;
use texture::{Texture2dArray, RawImage2d, TextureCreationError};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub enum TextureLoadingError {
	Io(std::io::Error),
	Image(image::error::ImageError),
	TextureCreation(TextureCreationError),
	InvalidConfigArgNumber(usize),
	UnknownResourceName(usize, String),
	DuplicateResource(usize, String),
	InvalidResourceType(usize, String),
	InconsistantTextureSize {
		wanted: (usize, usize),
		got: (usize, usize),
		file: PathBuf,
	}
}

impl From<std::io::Error> for TextureLoadingError {
	fn from(other: std::io::Error) -> Self {
		TextureLoadingError::Io(other)
	}
}

impl From<TextureCreationError> for TextureLoadingError {
	fn from(other: TextureCreationError) -> Self {
		TextureLoadingError::TextureCreation(other)
	}
}

impl From<image::error::ImageError> for TextureLoadingError {
	fn from(other: image::error::ImageError) -> Self {
		TextureLoadingError::Image(other)
	}
}

macro_rules! make_textures {
	($($name:ident: $path:expr),*,) => {
		#[derive(Clone, Copy)]
		#[repr(usize)]
		pub enum Texture {
			$($name,)*
			NTextures,
		}

		fn string_to_texture_variant(string: &str) -> Option<Texture> {
			match string {
				$(
					stringify!($name) => Some(Texture::$name),
				)*
				_ => None,
			}
		}
	}
}

make_textures! {
	Home: "home",
	Human: "human",
	HappyHome: "happy_home",
	Cake: "cake",
	SadHome: "sad_home",
	Player: "player",
	HumanWithGoop: "human_with_goop",
	CakeWithGoop: "cake_with_goop",
	BucketOfGoop: "bucket_of_goop",
	FloorMap: "floor_map",
	GoopMap: "goop_map",
	IceMap: "ice_map",
	VoidMap: "void_map",
	Grass: "grass",
	FlowerMap: "flower_map",
	MossyMap: "mossy_map",
}

#[derive(Clone, Copy, Debug)]
struct TextureMetaData {
	index: usize,
	n_textures: usize,
}

pub struct Textures {
	pub atlas: Texture2dArray,
	metadata: Vec<TextureMetaData>,
} 

impl Textures {
	pub fn load(path: impl AsRef<Path>, display: &Display)
		-> Result<Textures, TextureLoadingError> 
	{
		let mut metadata: Vec<Option<TextureMetaData>> = vec![None; Texture::NTextures as usize];
		let mut texture_files: Vec<PathBuf> = Vec::new();

		let mut parts = Vec::new();
		let contents = fs::read_to_string(path)?;
		for (line_num, line) in contents.lines()
			.enumerate()
			.map(|(i, v)| (i, v.trim()))
			.filter(|(_, v)| v.len() != 0 && !v.starts_with("//")) 
		{
			parts.clear();
			line.split(';').map(|v| v.trim()).for_each(|v| parts.push(v));

			let [name, kind, path] = if let &[name, kind, path] 
				= parts.as_slice() 
			{
				[name, kind, path]
			}else {
				return Err(TextureLoadingError::InvalidConfigArgNumber(line_num));
			};

			let texture = string_to_texture_variant(name)
				.ok_or_else(|| TextureLoadingError::UnknownResourceName(
					line_num, 
					format!("{}", name),
				))?;

			if metadata[texture as usize].is_some() {
				return Err(TextureLoadingError::DuplicateResource(
						line_num,
						format!("{}", name),
				));
			}

			match kind {
				"texture" => {
					metadata[texture as usize] = Some(TextureMetaData {
						index: texture_files.len(),
						n_textures: 1,
					});
					texture_files.push(path.into());
				}
				"map" => {
					metadata[texture as usize] = Some(TextureMetaData {
						index: texture_files.len(),
						n_textures: 8,
					});
					texture_files.push((path.to_string() + "0.png").into());
					texture_files.push((path.to_string() + "1.png").into());
					texture_files.push((path.to_string() + "2.png").into());
					texture_files.push((path.to_string() + "3.png").into());
					texture_files.push((path.to_string() + "4.png").into());
					texture_files.push((path.to_string() + "5.png").into());
					texture_files.push((path.to_string() + "6.png").into());
					texture_files.push((path.to_string() + "7.png").into());
				}
				_ => return Err(TextureLoadingError::InvalidResourceType(
					line_num, 
					format!("{}", kind),
				)),
			}
		}

		let mut unwrapped_metadata = Vec::with_capacity(metadata.len());
		for (i, element) in metadata.into_iter().enumerate() {
			if let Some(element) = element {
				unwrapped_metadata.push(element);
			} else {
				println!("WARNING: Incomplete assets.txt, resource number {} is not defined", i);
				unwrapped_metadata.push(TextureMetaData {
					index: 0,
					n_textures: 1,
				});
			}
		}

		// Load all the textures that were queued earlier
		let mut size = None;
		let mut loaded_textures = Vec::with_capacity(texture_files.len());
		for texture_file in texture_files {
			let image = image::open(&texture_file)?.into_rgba();
			let (width, height) = image.dimensions();

			if let Some(size) = size {
				if (width, height) != size {
					return Err(TextureLoadingError::InconsistantTextureSize {
						got: (width as usize, height as usize),
						wanted: (size.0 as usize, size.1 as usize),
						file: texture_file,
					});
				}
			} else {
				size = Some((width, height));
			}

			let raw = 
				RawImage2d::from_raw_rgba(image.into_raw(), (width, height));
			loaded_textures.push(raw);
			println!("Loaded texture '{:?}'", texture_file);
		}

		let texture_array = Texture2dArray::new(
			display,
			loaded_textures,
		)?;

		Ok(Textures {
			atlas: texture_array,
			metadata: unwrapped_metadata,
		})
	}

	pub fn get_uv(&self, texture: Texture) -> UVCoords {
		let metadata = self.metadata[texture as usize];
		UVCoords {
			left: 0.0,
			right: 1.0,
			bottom: 1.0,
			top: 0.0,
			texture: metadata.index as f32,
		} 
	} 

	pub fn get_tilemap_uv(&self, texture: Texture, horizontal: bool, vertical: bool, diagonal: bool) -> UVCoords {
		let metadata = self.metadata[texture as usize];

		if metadata.n_textures >= 8 {
			UVCoords {
				left: 0.0,
				right: 1.0,
				bottom: 1.0,
				top: 0.0,
				texture: metadata.index as f32 + 
					if horizontal { 4.0 } else { 0.0 } + 
					if vertical { 2.0 } else { 0.0 } +
					if diagonal { 1.0 } else { 0.0 },
			}
		} else {
			UVCoords {
				left: 0.0,
				right: 1.0,
				bottom: 1.0,
				top: 0.0,
				texture: metadata.index as f32,
			} 
		} 
	} 
}

#[derive(Clone, Copy, Debug)]
pub struct UVCoords {
	pub left: f32,
	pub right: f32,
	pub top: f32,
	pub bottom: f32,
	pub texture: f32,
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
			texture: self.texture,
		}
	}
}
