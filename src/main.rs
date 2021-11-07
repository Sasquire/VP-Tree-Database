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

#[macro_use]
extern crate rocket;
use clap::App;
use clap::Arg;

#[rocket::main]
async fn main() {
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
		.arg(
			Arg::with_name("server")
				.short("s")
				.long("server")
				.help("Starts a web-server on localhost"),
		)
		.get_matches();

	metadata_database::initialize_database();

	// TODO threaded insert where features are found
	// on threads and then inserts are done on a single thread

	if matches.value_of("add_image").is_some() {
		let image_path = matches.value_of("add_image").unwrap();
		println!("should add image {}", image_path);
		add::add_image_to_database(image_path);
	} else if matches.value_of("find_image").is_some() {
		let image_path = matches.value_of("find_image").unwrap();
		let k = get_k_from_cli(matches.value_of("k_nearest_neighbors"));
		search::rank_all_features_from_database(image_path, k);
	} else if matches.value_of("python_binary").is_some() {
		let python_binary = matches.value_of("python_binary").unwrap();
		println!("should merge binary {}", python_binary);
		add::add_python_binary_to_database(python_binary);
	} else if matches.value_of("print").is_some() {
		let print_path = matches.value_of("print").unwrap();
		println!("should print {}", print_path);
		features_database::print_path(String::from(print_path));
	} else if matches.occurrences_of("server") > 0 {
		network::start().await.unwrap();
	} else {
		println!("doing nothing");
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

mod search {
	use crate::feature_tree::search_result::SearchResult;
	use crate::features::feature_description::FeatureDescription;
	use crate::metadata_database::KeypointMetadata;

	pub fn rank_all_features_from_database(file_path: &str, number_of_neighbors: usize) {
		let image_features = crate::extract_from_image::get_features_from_image_path(file_path)
			.into_iter()
			.map(|e| e.description)
			.collect();
		for i in &image_features {
			println!("{:?}", i);
		}

		let results = search_for_all_descriptions(image_features, number_of_neighbors);
		for (id, (comparisons, search_results)) in results.into_iter().enumerate() {
			println!(
				"{:>5} Found {:>6} results in {:>13} comparisons",
				id,
				search_results.len(),
				comparisons
			);

			println!("input  rank distance                              md5 file-ext frame file-uuid          uuid        x        y   size  angle      response octave");

			for (counter, (result, metadata)) in search_results.into_iter().enumerate() {
				println!("{:>5} {:>5} {:>8} {:>32} {:>8} {:>5} {:>9} {:>13} {:>8.2} {:>8.2} {:>6.2} {:>6.2} {:>13.10} {:>6}",
					id,
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
	}

	type CountedSearchResult = (u64, Vec<(SearchResult, KeypointMetadata)>);

	pub fn search_for_all_descriptions(
		descriptions: Vec<FeatureDescription>,
		number_of_neighbors: usize,
	) -> Vec<CountedSearchResult> {
		let mut results = vec![];

		let mut threads = vec![];
		for description in descriptions {
			if crate::constants::THREADED_SEARCH {
				threads.push(std::thread::spawn(move || {
					search_for_description(description, number_of_neighbors)
				}));
			} else {
				results.push(search_for_description(description, number_of_neighbors));
			}
		}

		if crate::constants::THREADED_SEARCH {
			results = threads
				.into_iter()
				.map(|e| e.join().unwrap())
				.collect::<Vec<CountedSearchResult>>();
		}

		return results;
	}

	pub fn search_for_description(
		description: FeatureDescription,
		number_of_neighbors: usize,
	) -> CountedSearchResult {
		let (comparisons, results) = crate::features_database::find_feature_description_in_database(
			description,
			number_of_neighbors,
		);

		let metadata_list: Vec<KeypointMetadata> = results
			.iter()
			.map(|e| crate::metadata_database::find_metadata_from_uuid(e.get_result_uuid()))
			.collect();

		let pairs = results.into_iter().zip(metadata_list.into_iter()).collect();

		return (comparisons, pairs);
	}
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

mod network {
	use crate::features::feature_description::FeatureDescription;
	use crate::metadata_database::KeypointMetadata;

	use rocket::http::ContentType;
	use rocket::response::content;
	use rocket::serde::json::json;
	use rocket::serde::json::Json;
	use rocket::serde::Deserialize;
	use rocket::serde::Serialize;
	use rocket::State;
	use std::fs;
	use std::sync::Arc;
	use std::sync::Mutex;
	use std::thread;

	#[derive(Deserialize)]
	#[serde(crate = "rocket::serde")]
	struct Message {
		open_cv_results: Vec<Descriptor>,
		k: u8,
	}

	#[derive(Deserialize)]
	#[serde(crate = "rocket::serde")]
	#[allow(dead_code)]
	struct Descriptor {
		angle: f32,
		class_id: i32,
		descriptor: Vec<u8>,
		octave: u8,
		response: f32,
		size: f32,
		x: f32,
		y: f32,
	}

	#[derive(Serialize)]
	#[serde(crate = "rocket::serde")]
	struct SearchPair {
		distance: u32,
		rank: usize,
		metadata: KeypointMetadata,
	}

	#[derive(Serialize)]
	#[serde(crate = "rocket::serde")]
	struct VectorResult {
		results: Vec<SearchPair>,
		id: usize,
		comparisons: u64,
	}

	struct IsProgramSearching {
		value: Arc<Mutex<u8>>,
	}

	pub async fn start() -> Result<(), rocket::Error> {
		let config = IsProgramSearching {
			value: Arc::new(Mutex::new(0)),
		};

		return rocket::build()
			.mount("/", routes![get_index, get_opencv, get_favicon, get_image_results])
			.manage(config)
			.register("/", catchers![not_found])
			.launch()
			.await;
	}

	#[post("/get_image_results.json", format = "json", data = "<message>")]
	async fn get_image_results(
		message: Json<Message>,
		state: &State<IsProgramSearching>,
	) -> content::Json<rocket::serde::json::Value> {
		let descriptors = message
			.open_cv_results
			.iter()
			.map(|e| FeatureDescription::new_from_vec(e.descriptor.clone()))
			.collect();

		// This is not clean or elegant, but under Kira's advisement, I am only
		// letting one search be performed at a time. This is to make sure that
		// attackers can not overload my system with attacks. It may provide a
		// poor user experience if multiple images are trying to be searched at
		// once and the program is CPU bound, but frankly I don't care.
		// TODO make this pretty
		let clone_arc = state.value.clone();
		let results = thread::spawn(move || {
			let mut mutex_data = clone_arc.lock().unwrap();
			*mutex_data = 1;
			let results =
				crate::search::search_for_all_descriptions(descriptors, message.k as usize);
			*mutex_data = 0;
			results
		})
		.join()
		.unwrap();

		return content::Json(json!({
			"results": results.into_iter().enumerate().map(|(id, (comparisons, list))| VectorResult {
				id: id,
				comparisons: comparisons,
				results: list.into_iter().enumerate().map(|(rank, (search_results, metadata))| SearchPair {
					rank: rank,
					distance: search_results.get_distance(),
					metadata: metadata
				}).collect::<Vec<SearchPair>>(),
			}).collect::<Vec<VectorResult>>()
		}));
	}

	#[get("/")]
	fn get_index() -> content::Html<String> {
		content::Html(
			fs::read_to_string("./UI/index.html").expect("Error reading the index.html file"),
		)
	}

	#[get("/opencv.js")]
	fn get_opencv() -> content::JavaScript<String> {
		content::JavaScript(
			fs::read_to_string("./UI/opencv.js").expect("Error reading the index.html file"),
		)
	}

	#[get("/favicon.ico")]
	fn get_favicon() -> content::Custom<Vec<u8>> {
		content::Custom(
			ContentType::AVIF,
			fs::read("./UI/favicon.ico").expect("Error reading the favicon file"),
		)
	}

	#[catch(404)]
	fn not_found() -> String {
		return String::from("404");
	}
}
