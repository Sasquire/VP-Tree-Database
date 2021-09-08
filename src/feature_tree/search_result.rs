use crate::features::uuid_description_pair::UUIDDescriptionPair;

#[derive(Debug)]
pub struct SearchResult {
	feature: UUIDDescriptionPair,
	distance: u32,
	comparisons: u64,
}

impl SearchResult {
	pub fn new(feature: UUIDDescriptionPair, distance: u32, comparisons: u64) -> SearchResult {
		return SearchResult {
			feature,
			distance,
			comparisons,
		};
	}

	pub fn distance_from_target(&self) -> u32 {
		return self.distance;
	}

	pub fn get_comparisons(&self) -> u64 {
		return self.comparisons;
	}

	pub fn get_result_uuid(&self) -> u64 {
		return self.feature.get_uuid();
	}

	pub fn combine_comparisons(a: &mut SearchResult, b: &mut SearchResult) {
		let total_comparisons = a.comparisons + b.comparisons;
		a.comparisons = total_comparisons;
		b.comparisons = total_comparisons;
	}
}
