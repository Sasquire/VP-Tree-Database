use crate::features::feature_description::FeatureDescription;
use crate::features::uuid_description_pair::UUIDDescriptionPair;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug)]
pub struct SearchResult {
	feature: UUIDDescriptionPair,
	distance: u32,
}

impl SearchResult {
	pub fn new(feature: UUIDDescriptionPair, distance: u32) -> SearchResult {
		return SearchResult { feature, distance };
	}

	pub fn get_distance(&self) -> u32 {
		return self.distance;
	}

	pub fn get_result_uuid(&self) -> u64 {
		return self.feature.get_uuid();
	}
}

impl Ord for SearchResult {
	fn cmp(&self, other: &Self) -> Ordering {
		return self.get_distance().cmp(&other.get_distance());
	}
}

impl PartialOrd for SearchResult {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for SearchResult {}

impl PartialEq for SearchResult {
	fn eq(&self, other: &Self) -> bool {
		return self.get_distance() == other.get_distance();
	}
}

pub struct SearchResultList {
	results: BinaryHeap<SearchResult>,
	max_features: usize,
	target: FeatureDescription,
	comparisons: u64,
}

impl SearchResultList {
	pub fn try_to_add(&mut self, to_add: &UUIDDescriptionPair) {
		self.comparisons += 1;

		let distance_to_target = self.target.distance(to_add.get_description());
		if self.results.len() < self.max_features {
			let result = SearchResult::new(to_add.clone(), distance_to_target);
			self.results.push(result);
		} else if distance_to_target < self.results.peek().unwrap().get_distance() {
			let result = SearchResult::new(to_add.clone(), distance_to_target);
			self.results.push(result);
			let _worst_node = self.results.pop().unwrap();
		} else {
			// No Reason to add this awful node
		}
	}

	pub fn distance_to_feature(&self, feature: &FeatureDescription) -> u32 {
		return self.target.distance(feature);
	}

	pub fn get_worst_distance_to_target(&self) -> u32 {
		return self.results.peek().unwrap().get_distance();
	}

	pub fn new(max_features: usize, target: FeatureDescription) -> SearchResultList {
		return SearchResultList {
			results: BinaryHeap::new(),
			max_features: max_features,
			target: target,
			comparisons: 0,
		};
	}

	pub fn get_comparisons(&self) -> u64 {
		return self.comparisons;
	}

	pub fn get_results(self) -> Vec<SearchResult> {
		return self.results.into_sorted_vec();
	}
}
