use crate::feature_tree::leaf_node::LeafNode;
use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResultList;
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

	fn find(&self, results: &mut SearchResultList) {
		// TODO is it possible to make this cleaner?
		// radius belongs to far
		let distance_to_vantage = results.distance_to_feature(&self.vantage);
		let is_near = distance_to_vantage < self.radius;

		match is_near {
			true => self.near.find(results),
			false => self.far.find(results),
		};

		let is_lucky = match is_near {
			true => results.get_worst_distance_to_target() + distance_to_vantage < self.radius,
			false => self.radius + results.get_worst_distance_to_target() <= distance_to_vantage,
		};

		if is_lucky == false {
			match is_near {
				true => self.far.find(results),
				false => self.near.find(results),
			};
		}
	}

	fn size(&self) -> u64 {
		return (*self.near).size() + (*self.far).size();
	}

	fn print(&self, depth: u32) {
		let padding = (0..depth).map(|_e| String::from(" ")).collect::<String>();
		println!(
			"{}{}, n={:12}, f={:12}",
			padding,
			self.size(),
			self.near.size(),
			self.far.size()
		);
		self.near.print(depth + 1);
		self.far.print(depth + 1);
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
		return split_leaf_with_median_radius(node, split_point_path);
	}
}

#[allow(dead_code)]
fn split_leaf_with_default_radius(node: &mut LeafNode, split_point_path: NodePath) -> Node {
	let mut new_node = InternalNode::new_empty();
	for pair in node.get_owned_features() {
		new_node.add(pair, split_point_path.clone());
	}
	return new_node;
}

// Does not produce a perfectly balanced tree, but because the radius is chosen
// from a sample of crate::constants::MAX_LEAF_NODE_SIZE nodes, it is a more
// inteligent guess of what a good radius would be. Having a more balanced tree
// should obviously be desired, but with the less-balanced version, there were
// issues where file names became too long.
// This method could be improved for speed, but runs fast enough for now.
fn split_leaf_with_median_radius(node: &mut LeafNode, split_point_path: NodePath) -> Node {
	let pairs = node.get_owned_features();
	let vantage = FeatureDescription::random_edge();
	let mut distances: Vec<u32> = pairs
		.iter()
		.map(|e| e.get_description().distance(&vantage))
		.collect();
	distances.sort();
	let median = distances[pairs.len() / 2];

	let mut near = LeafNode::new_empty();
	let mut far = LeafNode::new_empty();
	for pair in pairs {
		let distance = pair.get_description().distance(&vantage);
		if distance < median {
			near.add(pair, split_point_path.clone());
		} else {
			far.add(pair, split_point_path.clone());
		}
	}

	return Node::Internal(InternalNode {
		vantage: vantage,
		radius: median,
		near: Box::new(near),
		far: Box::new(far),
	});
}
