use crate::extract_from_image::PointOfInterest;
use crate::features::feature_description::FeatureDescription;
use crate::frame_info::FrameInfo;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::convert::TryInto;
use std::str;
use std::vec::Vec;

pub fn parse_python_binary(file_name: &str) -> Vec<(FrameInfo, Vec<PointOfInterest>)> {
	const ENTRY_SIZE: usize = 4096 + 16384 + 16384;
	let (size, contents) = read_file_to_binary(file_name);

	let mut results = vec![];
	for start in (0..size).step_by(ENTRY_SIZE) {
		let range = start..(start + ENTRY_SIZE);
		results.push(parse_python_block(&contents[range]));
	}

	return results;
}

fn parse_python_block(data: &[u8]) -> (FrameInfo, Vec<PointOfInterest>) {
	// Header Block   0..4096
	// Metadata Block 4096..20480
	// Vector Block   20480..36864
	let (num_features, frame_info) = parse_header_block(&data[0..48]);
	let metadata = parse_metadata_block(&data[4096..20480], num_features);
	let vectors = parse_vector_block(&data[20480..36864], num_features);
	let points_of_interest = metadata
		.into_iter()
		.zip(vectors.into_iter())
		.map(|(m, v)| PointOfInterest {
			metadata: m,
			description: v,
		})
		.collect();
	return (frame_info, points_of_interest);

	fn parse_header_block(data: &[u8]) -> (u32, FrameInfo) {
		let md5 = String::from(str::from_utf8(&data[0..32]).unwrap());
		let ext = String::from(str::from_utf8(&data[32..40]).unwrap().trim());
		let frame_index = u32::from_le_bytes(data[40..44].try_into().unwrap()) as u64;
		let number_features = u32::from_le_bytes(data[44..48].try_into().unwrap());
		return (number_features, FrameInfo::new(md5, ext, frame_index));
	}

	fn parse_vector_block(data: &[u8], num_vectors: u32) -> Vec<FeatureDescription> {
		let mut result = vec![];
		// There was a weird glitch where one of the files had >500 features
		for i in 0..500.min(num_vectors as usize) {
			let start = i * 32;
			let range = start..(start + 32);
			result.push(FeatureDescription::new_from_vec(data[range].to_vec()));
		}
		return result;
	}

	fn parse_metadata_block(data: &[u8], num_vectors: u32) -> Vec<opencv::core::KeyPoint> {
		let mut result = vec![];
		// There was a weird glitch where one of the files had >500 features
		for i in 0..500.min(num_vectors as usize) {
			let start = i * 32;
			let x = f32::from_le_bytes(data[(start + 0)..(start + 4)].try_into().unwrap());
			let y = f32::from_le_bytes(data[(start + 4)..(start + 8)].try_into().unwrap());
			let size = f32::from_le_bytes(data[(start + 8)..(start + 12)].try_into().unwrap());
			let angle = f32::from_le_bytes(data[(start + 12)..(start + 16)].try_into().unwrap());
			let response = f32::from_le_bytes(data[(start + 16)..(start + 20)].try_into().unwrap());
			let octave = u32::from_le_bytes(data[(start + 20)..(start + 24)].try_into().unwrap());

			result.push(opencv::core::KeyPoint {
				pt: opencv::core::Point_::new(x, y),
				size: size,
				angle: angle,
				response: response,
				octave: octave as i32,
				class_id: 1,
			});
		}
		return result;
	}
}

fn read_file_to_binary(file_name: &str) -> (usize, Vec<u8>) {
	let file = File::open(file_name).unwrap();
	let mut buf_reader = BufReader::new(file);
	let mut contents = vec![];
	let size = buf_reader.read_to_end(&mut contents).unwrap();
	return (size, contents);
}
