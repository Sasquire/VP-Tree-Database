use crate::feature_tree::node::Node;
use crate::feature_tree::node::TreeNode;
use crate::feature_tree::node_path::NodePath;
use crate::feature_tree::search_result::SearchResult;
use crate::features::feature_description::FeatureDescription;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

pub fn insert_description_vec_into_database(description_vec: Vec<UUIDDescriptionPair>) {
	let total = description_vec.len();

	let mut root_node = Node::get_root_node();
	for (counter, pair) in description_vec.into_iter().enumerate() {
		if counter % 1000000 == 0 {
			println!("Adding node {} out of {}", counter, total);
		}
		root_node.add(pair, NodePath::new_empty());
	}
}

pub fn find_feature_description_in_database(to_find: &FeatureDescription) -> SearchResult {
	return Node::get_root_node().find(to_find);
}

pub fn print_path(path: String) {
	Node::get_file_as_root(path).print(0);
}
