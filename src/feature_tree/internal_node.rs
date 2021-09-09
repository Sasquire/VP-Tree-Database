use crate::feature_tree::leaf_node::LeafNode;
use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResult;
use crate::features::feature_description::FeatureDescription;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

use std::convert::TryInto;

#[derive(Clone)]
pub struct InternalNode {
	vantage: FeatureDescription,
	radius: u32,
	near: Box<Node>,
	far: Box<Node>,
}

impl TreeNode for InternalNode {
	fn new_empty() -> Node {
		return Node::Internal(InternalNode {
			vantage: FeatureDescription::random_edge(),
			radius: crate::constants::AVERAGE_EDGE_FEATURE_DISTANCE,
			near: Box::new(Node::new_empty()),
			far: Box::new(Node::new_empty()),
		});
	}

	fn add(&mut self, to_add: UUIDDescriptionPair, mut current_path: NodePath) -> bool {
		if to_add.get_description().distance(&self.vantage) < self.radius {
			current_path.add_direction(crate::constants::NEAR_KEY);
			return (*self.near).add(to_add, current_path);
		} else {
			current_path.add_direction(crate::constants::FAR_KEY);
			return (*self.far).add(to_add, current_path);
		}
	}

	fn find(&self, to_find: &FeatureDescription) -> SearchResult {
		// TODO is it possible to make this cleaner?
		// radius belongs to far
		let distance_to_vantage = self.vantage.distance(to_find);
		let is_near = distance_to_vantage < self.radius;

		if is_near {
			let mut near_guess = self.near.find(to_find);
			let is_lucky_guess =
				near_guess.distance_from_target() + distance_to_vantage < self.radius;
			if is_lucky_guess {
				return near_guess;
			} else {
				let mut far_guess = self.far.find(to_find);
				SearchResult::combine_comparisons(&mut near_guess, &mut far_guess);
				if near_guess.distance_from_target() < far_guess.distance_from_target() {
					return near_guess;
				} else {
					return far_guess;
				}
			}
		} else {
			// radius belongs to far
			let mut far_guess = self.far.find(to_find);
			let is_lucky_guess =
				self.radius + far_guess.distance_from_target() <= distance_to_vantage;
			if is_lucky_guess {
				return far_guess;
			} else {
				let mut near_guess = self.near.find(to_find);
				SearchResult::combine_comparisons(&mut near_guess, &mut far_guess);
				if near_guess.distance_from_target() < far_guess.distance_from_target() {
					return near_guess;
				} else {
					return far_guess;
				}
			}
		}
	}

	fn size(&self) -> u64 {
		return (*self.near).size() + (*self.far).size();
	}

	fn to_binary(&self) -> Vec<u8> {
		let mut results = vec![];
		results.append(
			&mut crate::constants::INTERNAL_NODE_SIGNATURE
				.as_bytes()
				.to_vec(),
		);
		results.append(&mut self.radius.to_le_bytes().to_vec());
		results.append(&mut self.vantage.to_binary());

		let mut near_binary = self.near.to_binary();
		results.append(&mut (near_binary.len() as u64).to_le_bytes().to_vec());
		results.append(&mut near_binary);

		let mut far_binary = self.far.to_binary();
		results.append(&mut (far_binary.len() as u64).to_le_bytes().to_vec());
		results.append(&mut far_binary);

		return results;
	}

	fn from_binary(binary: &[u8]) -> Node {
		let _node_type = &binary[0..4];
		let radius = u32::from_le_bytes(binary[4..8].try_into().expect("Slice has bad length"));
		let vantage = FeatureDescription::from_binary(&binary[8..40]);

		let near_length =
			u64::from_le_bytes(binary[40..48].try_into().expect("Slice has bad length"));
		let near_end = (48 + near_length) as usize;
		let near_block = Node::from_binary(&binary[48..near_end]);

		let far_length_range = near_end..(near_end + 8);
		let far_length = u64::from_le_bytes(
			binary[far_length_range]
				.try_into()
				.expect("Slice has bad length"),
		);
		let far_range = (near_end + 8)..(near_end + 8 + far_length as usize);
		let far_block = Node::from_binary(&binary[far_range]);

		return Node::Internal(InternalNode {
			radius: radius,
			vantage: vantage,
			near: Box::new(near_block),
			far: Box::new(far_block),
		});
	}
}

impl InternalNode {
	pub fn new_from_leaf(node: &mut LeafNode, split_point_path: NodePath) -> Node {
		return split_leaf_with_default_radius(node, split_point_path);
	}
}

fn split_leaf_with_default_radius(node: &mut LeafNode, split_point_path: NodePath) -> Node {
	let mut new_node = InternalNode::new_empty();
	for pair in node.get_owned_features() {
		new_node.add(pair, split_point_path.clone());
	}
	return new_node;
}
