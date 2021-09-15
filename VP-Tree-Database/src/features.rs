pub mod feature_description {
	use rand_chacha::ChaCha8Rng;
	use rand_core::RngCore;
	use rand_core::SeedableRng;
	use std::iter::Iterator;

	#[derive(Clone, Debug)]
	pub struct FeatureDescription {
		data: [u8; crate::constants::FEATURE_DESCRIPTION_LENGTH],
	}

	impl FeatureDescription {
		pub fn new(data: [u8; crate::constants::FEATURE_DESCRIPTION_LENGTH]) -> FeatureDescription {
			return FeatureDescription { data: data };
		}

		pub fn new_from_vec(input_vec: Vec<u8>) -> FeatureDescription {
			let mut data = empty_data();
			for i in 0..crate::constants::FEATURE_DESCRIPTION_LENGTH {
				data[i] = *input_vec
					.get(i)
					.expect("Converting Vec to FeatureDescription failed");
			}
			return FeatureDescription { data };
		}

		pub fn distance(&self, other: &FeatureDescription) -> u32 {
			// 80% of the program is in this function. It is slow because
			// of memory access not because this part is slow.

			/*
			let mut sum = 0;
			for (first, second) in self.data.iter().zip(other.data.iter()) {
				sum += ((*first as i32) - (*second as i32)) * ((*first as i32) - (*second as i32));
			}
			return sum as u32;
			*/

			return self
				.data
				.iter()
				.zip(other.data.iter())
				.map(|(&x, &y)| (x as i32, y as i32))
				.map(|(x, y)| (x - y) * (x - y))
				.reduce(|a, b| a + b)
				.unwrap() as u32;
		}

		#[allow(dead_code)]
		pub fn random() -> FeatureDescription {
			let mut data = empty_data();
			for i in 0..crate::constants::FEATURE_DESCRIPTION_LENGTH {
				data[i] = rand::random();
			}
			return FeatureDescription { data };
		}

		pub fn random_edge() -> FeatureDescription {
			let mut data = empty_data();
			for i in 0..crate::constants::FEATURE_DESCRIPTION_LENGTH {
				data[i] = if rand::random() { 255 } else { 0 };
			}
			return FeatureDescription { data };
		}

		#[allow(dead_code)]
		pub fn seeded_random(seed: u64) -> impl Iterator<Item = FeatureDescription> {
			let mut rng = ChaCha8Rng::seed_from_u64(seed);
			return (0..).map(move |_x| {
				let mut data = empty_data();
				for i in 0..crate::constants::FEATURE_DESCRIPTION_LENGTH {
					data[i] = rng.next_u32() as u8
				}
				FeatureDescription::new(data)
			});
		}

		pub fn to_binary(&self) -> Vec<u8> {
			return self.data.to_vec();
		}

		pub fn from_binary(binary: &[u8]) -> FeatureDescription {
			let mut data = empty_data();
			for i in 0..crate::constants::FEATURE_DESCRIPTION_LENGTH {
				data[i] = binary[i];
			}
			return FeatureDescription::new(data);
		}
	}

	fn empty_data() -> [u8; crate::constants::FEATURE_DESCRIPTION_LENGTH] {
		return [0; crate::constants::FEATURE_DESCRIPTION_LENGTH];
	}
}

pub mod uuid_description_pair {
	use crate::features::feature_description::FeatureDescription;
	use std::convert::TryInto;

	#[derive(Clone, Debug)]
	pub struct UUIDDescriptionPair {
		description: FeatureDescription,
		uuid: u64,
	}

	impl UUIDDescriptionPair {
		pub fn new(uuid: u64, description: FeatureDescription) -> UUIDDescriptionPair {
			return UUIDDescriptionPair { uuid, description };
		}

		pub fn get_description(&self) -> &FeatureDescription {
			return &self.description;
		}

		pub fn get_uuid(&self) -> u64 {
			return self.uuid;
		}

		pub fn to_binary(&self) -> Vec<u8> {
			let mut results = vec![];
			results.append(&mut self.uuid.to_le_bytes().to_vec());
			results.append(&mut self.description.to_binary());
			return results;
		}

		pub fn from_binary(binary: &[u8]) -> UUIDDescriptionPair {
			let uuid = u64::from_le_bytes(binary[0..8].try_into().expect("Slice has bad length"));
			let description = FeatureDescription::from_binary(&binary[8..40]);
			return UUIDDescriptionPair::new(uuid, description);
		}
	}
}
