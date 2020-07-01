use std::sync::mpsc;
use std::fs;

enum SoundMessage {
	Play(SoundId, f32),
}

pub enum SoundId {
	Music = 0,
}

#[derive(Clone)]
pub struct Sounds {
	channel: mpsc::Sender<SoundMessage>,
}

impl Sounds {
	/// Loads sounds into a Sounds struct.
	///
	/// Warning: Leaks memory!!! Only call this once please.
	/// If you want several of them, call ``.clone()``
	pub fn load() -> Result<Sounds, String> {
		let mut files = Vec::new();
		for sound_path in &[
			"assets/music.wav",
		] {
			let file = fs::File::open("assets/music.wav").unwrap();
			files.push(
				&*Box::leak(
					fs::read(sound_path)
						.map_err(|v| format!("{:?}", v))?
						.into_boxed_slice()
				)
			);
		}

		// Creating this thread is a silly workaround for a bug in
		// someone elses library, sigh...
		//
		// I would have preferred to store the sounds in the Sounds thing
		// and not do all this transmit/recieve stuff, but hey, at least
		// I don't have to figure out the type of the vector!
		let (sound_transmit, sound_recieve) = mpsc::channel();
		std::thread::spawn(move || {
			use std::fs::File;
			use std::io::BufReader;
			use rodio::Source;

			let device = match rodio::default_output_device() {
				Some(device) => device,
				None => {
					println!("Cannot find a default output device, you'll have \
						to play without sound");
					return;
				}
			};
			while let Ok(message) = sound_recieve.recv() {
				match message {
					SoundMessage::Play(id, volume) => {
						let source = rodio::Decoder::new(
							std::io::Cursor::new(files[id as usize])
						).unwrap();
						rodio::play_raw(
							&device, 
							source
								.convert_samples()
								.amplify(volume)
						);
					}
				}
			}
		});

		Ok(Sounds {
			channel: sound_transmit,
		})
	}

	pub fn play(&self, sound: SoundId, volume: f32) {
		match self.channel.send(SoundMessage::Play(sound, volume)) {
			Ok(()) => (),
			Err(val) => {
				println!("Something went wrong with the sound, no more sounds \
					can play!");
			}
		}
	}
}
