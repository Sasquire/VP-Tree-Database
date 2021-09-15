use std::path::Path;

pub struct FrameInfo {
	md5: String,
	ext: String,
	index: u64,
}

impl FrameInfo {
	pub fn new_from_static_image_path(file_path: &str) -> FrameInfo {
		let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
		let splits = file_name.split(".").collect::<Vec<_>>();

		let md5 = String::from(*splits.get(0).unwrap());
		let file_ext = String::from(*splits.get(1).unwrap());
		let frame_id = 0;

		return FrameInfo {
			md5: md5,
			ext: file_ext,
			index: frame_id,
		};
	}

	pub fn new(md5: String, ext: String, index: u64) -> FrameInfo {
		return FrameInfo {
			md5: md5,
			ext: ext,
			index: index,
		};
	}

	pub fn copy_md5(&self) -> String {
		self.md5.clone()
	}
	pub fn copy_ext(&self) -> String {
		self.ext.clone()
	}
	pub fn get_id(&self) -> u64 {
		self.index
	}
}
