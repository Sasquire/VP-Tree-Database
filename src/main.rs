mod constants;
mod features;
mod frame_info;

mod extract_from_image;

mod features_database;
mod metadata_database;
mod feature_tree {
	mod file_node;
	mod internal_node;
	mod leaf_node;
	pub mod node;

	pub mod node_path;
	pub mod search_result;
}

use clap::App;
use clap::Arg;

fn main() {
	let matches = App::new("My Test Program")
		.version(crate::constants::VERSION)
		.author(crate::constants::CONTACT_INFO)
		.about(crate::constants::ABOUT)
		//	.arg(Arg::with_name("root")
		//			 .short("r")
		//			 .long("root")
		//			 .takes_value(true)
		//			 .required(true)
		//			 .help("Filepath for the root of the tree to perform operations"))
		.arg(
			Arg::with_name("add_image")
				.short("a")
				.long("add_image")
				.takes_value(true)
				.help("Filepath to an image which should be added to the database"),
		)
		.arg(
			Arg::with_name("find_image")
				.short("f")
				.long("find_image")
				.takes_value(true)
				.help("Filepath to an image which should be searched for in the database"),
		)
		// .arg(Arg::with_name("num")
		//		.short("n")
		//		.long("number")
		//		.takes_value(true)
		//		.help("Five less than your favorite number"))
		.get_matches();

	metadata_database::initialize_database();

	if matches.value_of("add_image").is_some() {
		let image_path = matches.value_of("add_image").unwrap();
		println!("should add image {}", image_path);
		add_image::add_image_to_database(image_path);
	} else if matches.value_of("find_image").is_some() {
		let image_path = matches.value_of("find_image").unwrap();
		println!("should search image {}", image_path);
		rank_all_features_from_database(image_path);
	} else {
		println!("not adding an image");
	}
}

fn rank_all_features_from_database(file_path: &str) {
	let image_features = extract_from_image::get_features_from_image_path(file_path);
	for point_of_interest in image_features {
		let result =
			features_database::find_feature_description_in_database(&point_of_interest.description);
	//	println!(
	//		"Found a match with distance {} in {} comparisons. Match UUID is {}.",
	//		result.distance_from_target(),
	//		result.get_comparisons(),
	//		result.get_result_uuid()
	//	);
		let metadata = metadata_database::find_metadata_from_uuid(result.get_result_uuid());
		println!("{}", metadata.md5);
	}
}

mod add_image {
	use crate::extract_from_image;
	use crate::extract_from_image::PointOfInterest;
	use crate::features::uuid_description_pair::UUIDDescriptionPair;
	use crate::frame_info::FrameInfo;

	use crate::features_database;
	use crate::metadata_database;

	pub fn add_image_to_database(file_path: &str) {
		let image_features = extract_from_image::get_features_from_image_path(file_path);
		insert_features_into_databases(
			image_features,
			FrameInfo::new_from_static_image_path(file_path),
		);
	}

	fn insert_features_into_databases(points_of_interest: Vec<PointOfInterest>, frame: FrameInfo) {
		let uuids = get_uuids(points_of_interest.len());

		type VecsDescription = (Vec<(u64, opencv::core::KeyPoint)>, Vec<UUIDDescriptionPair>);
		let (metadata_vec, description_vec): VecsDescription = points_of_interest
			.into_iter()
			.zip(uuids)
			.map(|(poi, uuid)| {
				(
					(uuid, poi.metadata),
					UUIDDescriptionPair::new(uuid, poi.description),
				)
			})
			.unzip();

		metadata_database::insert_metadata_vec_into_database(metadata_vec, frame);
		features_database::insert_description_vec_into_database(description_vec);
	}

	fn get_uuids(how_many: usize) -> std::ops::Range<u64> {
		let max_uuid_stored = metadata_database::get_max_uuid();
		let min = max_uuid_stored + 1;
		let max = max_uuid_stored + 1 + how_many as u64;
		return min..max;
	}
}
