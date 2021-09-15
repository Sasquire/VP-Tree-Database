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

mod python_binary;

use clap::App;
use clap::Arg;

fn main() {
	let matches = App::new(crate::constants::APP_NAME)
		.version(crate::constants::VERSION)
		.author(crate::constants::CONTACT_INFO)
		.about(crate::constants::ABOUT)
		.long_about(crate::constants::LONG_ABOUT)
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
		.arg(
			Arg::with_name("k_nearest_neighbors")
				.short("k")
				.long("k_nearest_neighbors")
				.takes_value(true)
				.help("Positive integer for the maximum number of neighbors (only used when using -f)"),
		)
		.arg(
			Arg::with_name("python_binary")
				.long("python_binary")
				.takes_value(true)
				.help("Filepath to a binary file containing ORB output that was extracted with python long ago"),
		)
		.arg(
			Arg::with_name("print")
				.long("print")
				.takes_value(true)
				.help("Filepath to a binary file that should be printed"),
		)
		.get_matches();

	metadata_database::initialize_database();

	if matches.value_of("add_image").is_some() {
		let image_path = matches.value_of("add_image").unwrap();
		println!("should add image {}", image_path);
		add::add_image_to_database(image_path);
	} else if matches.value_of("find_image").is_some() {
		let image_path = matches.value_of("find_image").unwrap();
		let k = get_k_from_cli(matches.value_of("k_nearest_neighbors"));
		println!("should search image {}", image_path);
		rank_all_features_from_database(image_path, k);
	} else if matches.value_of("python_binary").is_some() {
		let python_binary = matches.value_of("python_binary").unwrap();
		println!("should merge binary {}", python_binary);
		add::add_python_binary_to_database(python_binary);
	} else if matches.value_of("print").is_some() {
		let print_path = matches.value_of("print").unwrap();
		println!("should print {}", print_path);
		features_database::print_path(String::from(print_path));
	} else {
		println!("not adding an image");
	}
}

fn rank_all_features_from_database(file_path: &str, number_of_neighbors: usize) {
	let image_features = extract_from_image::get_features_from_image_path(file_path);

	let mut threads = vec![];
	for (id, point_of_interest) in image_features.into_iter().enumerate() {
		if crate::constants::THREADED_SEARCH {
			threads.push(std::thread::spawn(move || {
				rank_feature_from_database(id, point_of_interest, number_of_neighbors)
			}));
		} else {
			rank_feature_from_database(id, point_of_interest, number_of_neighbors);
		}
	}
	threads.into_iter().for_each(|e| e.join().unwrap());
}

fn rank_feature_from_database(
	thread_id: usize,
	point_of_interest: extract_from_image::PointOfInterest,
	number_of_neighbors: usize,
) {
	let (comparisons, results) = features_database::find_feature_description_in_database(
		point_of_interest.description,
		number_of_neighbors,
	);
	println!(
		"{:>5} Found {:>6} results in {:>13} comparisons                                      {:>8.2} {:>8.2} {:>6.2} {:>6.2} {:>13.10} {:>6}",
		thread_id,
		results.len(),
		comparisons,
		point_of_interest.metadata.pt.x,
		point_of_interest.metadata.pt.y,
		point_of_interest.metadata.size,
		point_of_interest.metadata.angle,
		point_of_interest.metadata.response,
		point_of_interest.metadata.octave
	);

	println!("input rank distance                              md5 file-ext frame file-uuid          uuid        x        y   size  angle      response octave");
	for (counter, result) in results.iter().enumerate() {
		let metadata = metadata_database::find_metadata_from_uuid(result.get_result_uuid());
		println!("{:>5} {:>5} {:>8} {:>32} {:>8} {:>5} {:>9} {:>13} {:>8.2} {:>8.2} {:>6.2} {:>6.2} {:>13.10} {:>6}",
			thread_id,
			counter,
			result.get_distance(),
			metadata.md5,
			metadata.file_ext,
			metadata.frame_id,
			metadata.file_uuid,
			metadata.uuid,
			metadata.x,
			metadata.y,
			metadata.size,
			metadata.angle,
			metadata.response,
			metadata.octave,
		);
	}
}

fn get_k_from_cli(k: Option<&str>) -> usize {
	if k.is_some() {
		let k = k.unwrap().parse();
		if k.is_ok() {
			return crate::constants::MAX_K_VALUE.min(1.max(k.unwrap()));
		}
	}

	return crate::constants::DEFAULT_K;
}

mod add {
	use crate::extract_from_image;
	use crate::extract_from_image::PointOfInterest;
	use crate::features::uuid_description_pair::UUIDDescriptionPair;
	use crate::frame_info::FrameInfo;

	use crate::features_database;
	use crate::metadata_database;

	use crate::python_binary;

	pub fn add_image_to_database(file_path: &str) {
		let image_features = extract_from_image::get_features_from_image_path(file_path);
		insert_metadata_and_description_to_database(assign_uuids_to_list(vec![(
			FrameInfo::new_from_static_image_path(file_path),
			image_features,
		)]));
	}

	pub fn add_python_binary_to_database(file_path: &str) {
		let files = python_binary::parse_python_binary(file_path);
		let files = assign_uuids_to_list(files);
		insert_metadata_and_description_to_database(files);
	}

	fn insert_metadata_and_description_to_database(list: FeaturesWithUUID) {
		let (metadata_list, description_pairs) = list;

		if crate::constants::THREADED_INSERT {
			let sqlite_handle = std::thread::spawn(|| {
				metadata_database::insert_meta_data_pair_vec_to_database(metadata_list)
			});
			let vp_tree_handle = std::thread::spawn(|| {
				features_database::insert_description_vec_into_database(description_pairs)
			});

			sqlite_handle.join().unwrap();
			vp_tree_handle.join().unwrap();
		} else {
			metadata_database::insert_meta_data_pair_vec_to_database(metadata_list);
			features_database::insert_description_vec_into_database(description_pairs);
		}
	}

	type FrameMetaDataPair = (FrameInfo, Vec<(u64, opencv::core::KeyPoint)>);
	type FeaturesWithUUID = (Vec<FrameMetaDataPair>, Vec<UUIDDescriptionPair>);
	fn assign_uuids_to_list(list: Vec<(FrameInfo, Vec<PointOfInterest>)>) -> FeaturesWithUUID {
		let mut uuid_iterator = (metadata_database::get_max_uuid() + 1)..;
		let mut metadata_frame_list = vec![];
		let mut all_descriptions = vec![];

		for (frame, poi_list) in list {
			// Only include static images currently
			if frame.get_id() != 0 {
				continue;
			}

			let mut metadata_vec = vec![];
			for poi in poi_list {
				let uuid = uuid_iterator.next().unwrap();
				metadata_vec.push((uuid, poi.metadata));
				all_descriptions.push(UUIDDescriptionPair::new(uuid, poi.description));
			}
			metadata_frame_list.push((frame, metadata_vec));
		}

		return (metadata_frame_list, all_descriptions);
	}
}
