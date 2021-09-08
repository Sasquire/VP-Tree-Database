use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResult;
use crate::features::feature_description::FeatureDescription;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

use atomicwrites::AllowOverwrite;
use atomicwrites::AtomicFile;

#[derive(Clone)]
pub struct FileNode {
	path_in_tree: NodePath,
}

fn get_node_from_file(file_path: String) -> Node {
	let file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(file_path)
		.expect("Opening the VP database failed");
	let mut buf_reader = std::io::BufReader::new(file);
	let mut contents = vec![];
	let size = buf_reader.read_to_end(&mut contents).unwrap();
	let node = if size == 0 {
		Node::new_empty()
	} else {
		Node::from_binary(&contents)
	};
	return node;
}

fn overwrite_node_to_file(file_path: String, node: Node) {
	AtomicFile::new(file_path, AllowOverwrite)
		.write(|file| file.write_all(&node.to_binary()))
		.expect("Writing to atomic file failed");
}

impl FileNode {
	pub fn new_at_location(path_in_tree: NodePath) -> Node {
		return Node::File(FileNode {
			path_in_tree: path_in_tree,
		});
	}

	fn open_file(&self) -> Node {
		return get_node_from_file(self.path_in_tree.to_file_path_string());
	}

	fn save_file(&self, to_save: Node) {
		overwrite_node_to_file(self.path_in_tree.to_file_path_string(), to_save);
	}
}

impl TreeNode for FileNode {
	fn new_empty() -> Node {
		return Node::File(FileNode {
			path_in_tree: NodePath::new_empty(),
		});
	}

	fn add(&mut self, to_add: UUIDDescriptionPair, mut current_path: NodePath) -> bool {
		let mut file_data = self.open_file();
		current_path.add_direction(crate::constants::FILE_KEY);
		let file_changed = file_data.add(to_add.clone(), current_path);
		if file_changed {
			self.save_file(file_data);
		}

		return false;
	}

	fn find(&self, to_find: &FeatureDescription) -> SearchResult {
		return self.open_file().find(to_find);
	}

	fn size(&self) -> u64 {
		return self.open_file().size();
	}

	fn to_binary(&self) -> Vec<u8> {
		let mut results = vec![];
		results.append(&mut crate::constants::FILE_NODE_SIGNATURE.as_bytes().to_vec());
		results.append(&mut self.path_in_tree.to_binary());
		return results;
	}

	fn from_binary(binary: &[u8]) -> Node {
		let _node_type = &binary[crate::constants::SIGNATURE_RANGE];
		let path = NodePath::from_binary(binary);
		return FileNode::new_at_location(path);
	}
}
