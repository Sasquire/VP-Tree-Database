use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResultList;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

#[derive(Clone)]
pub struct FileNode {
	path_in_tree: NodePath,
	file_contents: Option<Box<Node>>,
	has_changed: bool,
}

fn get_node_from_file(file_path: String) -> Node {
	//	let folder_path = Path::new(&file_path).parent().unwrap().to_str().unwrap();
	//	std::fs::create_dir_all(folder_path).unwrap();

	let file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(file_path)
		.expect("Opening a VP database file failed");
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

fn overwrite_node_to_file(file_path: String, data: Vec<u8>) {
	OpenOptions::new()
		.write(true)
		.create(true)
		.open(file_path)
		.expect("Opening a VP database file failed")
		.write_all(&data)
		.expect("Writing a VP database file failed");
}

impl FileNode {
	pub fn new_at_location(path_in_tree: NodePath) -> Node {
		return Node::File(FileNode {
			path_in_tree: path_in_tree,
			file_contents: None,
			has_changed: false,
		});
	}

	fn open(&mut self) {
		if self.file_contents.is_none() {
			let node = get_node_from_file(self.path_in_tree.to_file_path_string());
			self.file_contents = Some(Box::new(node));
		}
	}

	fn save(&mut self) {
		let node = self
			.file_contents
			.as_mut()
			.expect("Tried to save an empty file node");
		overwrite_node_to_file(self.path_in_tree.to_file_path_string(), node.to_binary());
		self.file_contents = None;
		self.has_changed = false;
	}

	fn open_temporarily(&self) -> Node {
		return get_node_from_file(self.path_in_tree.to_file_path_string());
	}
}

impl Drop for FileNode {
	fn drop(&mut self) {
		if self.has_changed {
			self.save();
		}
	}
}

impl TreeNode for FileNode {
	fn new_empty() -> Node {
		return FileNode::new_at_location(NodePath::new_empty());
	}

	fn add(&mut self, to_add: UUIDDescriptionPair, mut current_path: NodePath) -> bool {
		self.open();

		current_path.add_direction(crate::constants::FILE_KEY);
		let did_change = self
			.file_contents
			.as_mut()
			.expect("Tried to add node to file that was not open")
			.add(to_add, current_path);

		// TODO find better times to save file nodes
		if did_change == true {
			self.has_changed = true;
			//	if self.path_in_tree.size() > 10 {
			//		self.save();
			//	}
		}

		return false;
	}

	fn find(&self, results: &mut SearchResultList) {
		return self.open_temporarily().find(results);
	}

	fn size(&self) -> u64 {
		return self.open_temporarily().size();
	}

	fn print(&self, depth: u32) {
		let padding = (0..depth).map(|_e| String::from(" ")).collect::<String>();
		println!(
			"{}Entering new file {}",
			padding,
			self.path_in_tree.to_file_path_string()
		);
		self.open_temporarily().print(depth + 1);
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
