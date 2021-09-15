// Used this as a point of reference for how to use OpenCV in rust
// https://github.com/donkeyteethUX/abow/blob/09afd87afa856afb8f720a8942edcd32febc5a27/src/opencv_utils.rs

use crate::features::feature_description::FeatureDescription;

use opencv::core::KeyPoint;
use opencv::core::MatTrait;
use opencv::prelude::Feature2DTrait;

type CvImage = opencv::prelude::Mat;
type CvMat = opencv::core::Mat;

pub struct PointOfInterest {
	pub metadata: KeyPoint,
	pub description: FeatureDescription,
}

pub fn get_features_from_image_path(image_path: &str) -> Vec<PointOfInterest> {
	let image = load_image_path(image_path);
	let features = get_features_from_image(&image);
	return features;
}

fn get_features_from_image(image: &CvImage) -> Vec<PointOfInterest> {
	// If really want to edit the defaults later
	// https://docs.rs/opencv/0.53.1/opencv/features2d/trait.ORB.html#method.create
	let mut orb = <dyn opencv::features2d::ORB>::default().unwrap();

	let mask = CvMat::default();
	let mut keypoints = opencv::types::VectorOfKeyPoint::new();
	let mut descriptions = CvMat::default();
	orb.detect_and_compute(image, &mask, &mut keypoints, &mut descriptions, false)
		.expect("Computing keypoints for image failed");

	let descriptions = matrix_to_vec_of_descriptions(descriptions, keypoints.len() as i32, 32);

	let mut points_of_interest = vec![];
	for (keypoint, description) in keypoints.into_iter().zip(descriptions.into_iter()) {
		points_of_interest.push(PointOfInterest {
			metadata: keypoint,
			description: description,
		});
	}

	return points_of_interest;
}

fn matrix_to_vec_of_descriptions(
	matrix: CvImage,
	rows: i32,
	columns: i32,
) -> Vec<FeatureDescription> {
	let mut all_rows = vec![];
	for i in 0..rows {
		let mut this_row = vec![];
		for j in 0..columns {
			this_row.push(
				*matrix
					.at_2d::<u8>(i, j)
					.expect("Somehow accessed invalid index in image"),
			);
		}
		all_rows.push(FeatureDescription::new_from_vec(this_row));
	}
	return all_rows;
}

fn load_image_path(image_path: &str) -> CvImage {
	let image = opencv::imgcodecs::imread(
		image_path,
		opencv::imgcodecs::IMREAD_COLOR, // https://docs.rs/opencv/0.53.1/opencv/imgcodecs/enum.ImreadModes.html
	)
	.expect("Should error here but doesn't");

	// If the image cannot be read (because of missing file, improper permissions,
	// unsupported or invalid format), the function returns an empty matrix.
	if image.cols() == 0 && image.rows() == 0 {
		panic!("Image loaded but zero valued columns and rows mean error reading");
	}

	return image;
}
