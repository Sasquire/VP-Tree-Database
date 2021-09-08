use std::convert::TryInto;

#[derive(Clone)]
pub struct NodePath {
	path: Vec<u8>,
}

impl NodePath {
	pub fn new_empty() -> NodePath {
		return NodePath { path: vec![] };
	}

	pub fn add_direction(&mut self, direction: u8) {
		self.path.push(direction);
	}

	pub fn to_file_path_string(&self) -> String {
		let path_string: String = self.path.iter().map(|&e| e as char).collect();
		let file_name = String::from("vp_tree.") + &path_string + ".database";
		let file_path = String::from(crate::constants::DATABASE_FOLDER_PATH) + &file_name;
		return file_path;
	}

	pub fn should_split_to_new_file(&self) -> bool {
		let current_depth = self
			.path
			.iter()
			.filter(|&&e| e != crate::constants::FILE_KEY)
			.collect::<Vec<_>>()
			.len();
		let at_splitting_depth = current_depth % crate::constants::MAX_FILE_NODE_DEPTH == 0;
		let last_node_is_file = self
			.path
			.last()
			.or(Some(&crate::constants::UNUSED_KEY))
			.unwrap() == &crate::constants::FILE_KEY;
		return at_splitting_depth && last_node_is_file == false && current_depth != 0;
	}

	pub fn to_binary(&self) -> Vec<u8> {
		let mut results = vec![];
		results.append(&mut (self.path.len() as u64).to_le_bytes().to_vec());
		results.append(&mut self.path.iter().map(|e| *e).collect());
		return results;
	}

	pub fn from_binary(binary: &[u8]) -> NodePath {
		let _node_type = &binary[crate::constants::SIGNATURE_RANGE];
		let path_length =
			u64::from_le_bytes(binary[4..12].try_into().expect("Slice has bad length"));
		let path = binary[12..(12 + path_length) as usize]
			.iter()
			.map(|e| *e)
			.collect();

		return NodePath { path: path };
	}
}
