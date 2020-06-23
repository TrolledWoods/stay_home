mod level;
mod prelude {
	pub use glium::*;
}

use prelude::*;

fn main() {
	let level = r#"
	###
	H.#
	###
	"#;

	let level = level::Level::from_string(level).unwrap();
	println!("{:?}", level);
}
