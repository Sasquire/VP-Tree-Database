use crate::feature_tree::file_node::FileNode;
use crate::feature_tree::internal_node::InternalNode;
use crate::feature_tree::leaf_node::LeafNode;

use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResultList;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

#[derive(Clone)]
pub enum Node {
	Internal(InternalNode),
	Leaf(LeafNode),
	File(FileNode),
}

impl Node {
	pub fn get_root_node() -> Node {
		let mut root_path = NodePath::new_empty();
		root_path.add_direction(crate::constants::FILE_KEY);
		return FileNode::new_at_location(root_path);
	}

	pub fn get_file_as_root(file_path: String) -> Node {
		return FileNode::new_at_location(NodePath::from_file_path_string(file_path));
	}
}

// TODO am I using traits correctly?
// TODO optimize from_binary
// I find it unlikely that `find` can be optimized very well, but a substantial
// amount of time is spent building the nodes themselves. Not all of the node
// needs to be loaded into memory at the same time so if it can be made so that
// parts of nodes are only parsed when needed, it would speed up the time to
// find a feature.
pub trait TreeNode {
	fn new_empty() -> Node;
	fn add(&mut self, to_add: UUIDDescriptionPair, current_path: NodePath) -> bool;
	fn find(&self, results: &mut SearchResultList);
	fn size(&self) -> u64;

	fn print(&self, depth: u32);

	fn to_binary(&self) -> Vec<u8>;
	fn from_binary(binary: &[u8]) -> Node;
}

// TODO find a better way call these generic functions
impl TreeNode for Node {
	fn new_empty() -> Node {
		return LeafNode::new_empty();
	}

	fn add(&mut self, to_add: UUIDDescriptionPair, current_path: NodePath) -> bool {
		return match self {
			Node::Leaf(node) => {
				if current_path.should_split_to_new_file() {
					let new_node = FileNode::new_at_location(current_path.clone());
					let _old_node = std::mem::replace(self, new_node);
					self.add(to_add, current_path);
					true
				} else if should_split_to_internal_node(node.size()) {
					let new_node = InternalNode::new_from_leaf(node, current_path.clone());
					let _old_node = std::mem::replace(self, new_node);
					self.add(to_add, current_path);
					true
				} else {
					// Normal leaf node
					node.add(to_add, current_path)
				}
			}
			Node::Internal(node) => node.add(to_add, current_path),
			Node::File(node) => node.add(to_add, current_path),
		};

		fn should_split_to_internal_node(current_size: u64) -> bool {
			return current_size + 1 > crate::constants::MAX_LEAF_NODE_SIZE;
		}
	}

	fn find(&self, results: &mut SearchResultList) {
		match self {
			Node::Internal(node) => node.find(results),
			Node::Leaf(node) => node.find(results),
			Node::File(node) => node.find(results),
		}
	}

	fn size(&self) -> u64 {
		match self {
			Node::Internal(node) => node.size(),
			Node::Leaf(node) => node.size(),
			Node::File(node) => node.size(),
		}
	}

	fn to_binary(&self) -> Vec<u8> {
		match self {
			Node::Internal(node) => node.to_binary(),
			Node::Leaf(node) => node.to_binary(),
			Node::File(node) => node.to_binary(),
		}
	}

	fn print(&self, depth: u32) {
		match self {
			Node::Internal(node) => node.print(depth),
			Node::Leaf(node) => node.print(depth),
			Node::File(node) => node.print(depth),
		}
	}

	// TODO this looks ugly
	fn from_binary(binary: &[u8]) -> Node {
		let node_type = &binary[0..4];
		if binary_equals_data(node_type, crate::constants::LEAF_NODE_SIGNATURE) {
			return LeafNode::from_binary(binary);
		} else if binary_equals_data(node_type, crate::constants::INTERNAL_NODE_SIGNATURE) {
			return InternalNode::from_binary(binary);
		} else if binary_equals_data(node_type, crate::constants::FILE_NODE_SIGNATURE) {
			return FileNode::from_binary(binary);
		} else {
			panic!("Encountered unknown node of type {:?}", node_type);
		}

		fn binary_equals_data(binary: &[u8], data: &str) -> bool {
			return binary.iter().zip(data.as_bytes()).all(|(a, b)| a == b);
		}
	}
}
