use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResultList;
use crate::features::uuid_description_pair::UUIDDescriptionPair;
use std::convert::TryInto;

#[derive(Clone)]
pub struct LeafNode {
	features: Vec<UUIDDescriptionPair>,
}

impl LeafNode {
	pub fn get_owned_features(&mut self) -> Vec<UUIDDescriptionPair> {
		let mut stolen = vec![];
		std::mem::swap(&mut stolen, &mut self.features);
		return stolen;
	}
}

impl TreeNode for LeafNode {
	fn new_empty() -> Node {
		return Node::Leaf(LeafNode { features: vec![] });
	}

	fn add(&mut self, to_add: UUIDDescriptionPair, _current_path: NodePath) -> bool {
		// TODO maybe a check to ensure that nodes are not added as duplicates?
		self.features.push(to_add);
		return true;
	}

	// TODO does not support for k nearest points, only 1
	fn find(&self, results: &mut SearchResultList) {
		for pair in &self.features {
			results.try_to_add(pair);
		}
	}

	fn size(&self) -> u64 {
		return self.features.len() as u64;
	}

	fn print(&self, _depth: u32) {
		for _pair in &self.features {
			// TODO toggle for showing the values inside leaf nodes
			// println!("{:?}", pair);
		}
	}

	fn to_binary(&self) -> Vec<u8> {
		let mut results = vec![];
		results.append(&mut crate::constants::LEAF_NODE_SIGNATURE.as_bytes().to_vec());
		results.append(&mut self.size().to_le_bytes().to_vec());
		for pair in &self.features {
			results.append(&mut pair.to_binary());
		}
		return results;
	}

	fn from_binary(binary: &[u8]) -> Node {
		let _node_type = &binary[crate::constants::SIGNATURE_RANGE];
		let number_nodes =
			u64::from_le_bytes(binary[4..12].try_into().expect("Slice has bad length"));
		// TODO avoid using the new node and adding everything one by one
		let mut new_node = LeafNode::new_empty();
		for i in 0..number_nodes {
			let start = (12 + i * 40) as usize;
			let end = (start + 40) as usize;
			new_node.add(
				UUIDDescriptionPair::from_binary(&binary[start..end]),
				NodePath::new_empty(),
			);
		}
		return new_node;
	}
}
