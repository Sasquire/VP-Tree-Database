// TODO this whole file is ugly and needs to be cleaned up

use crate::frame_info::FrameInfo;

use opencv::core::KeyPoint;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::Statement;

pub fn initialize_database() {
	const CREATE_TABLE_FILES_STRING: &str = "CREATE TABLE IF NOT EXISTS files (
		file_uuid INTEGER PRIMARY KEY ON CONFLICT ABORT,
		md5 TEXT,
		file_ext TEXT,
		frame_id INTEGER,
		CONSTRAINT file_uniqueness UNIQUE (md5, file_ext, frame_id) ON CONFLICT ABORT
	)";

	const CREATE_TABLE_METADATA_STRING: &str = "CREATE TABLE IF NOT EXISTS metadata (
		uuid INTEGER PRIMARY KEY ON CONFLICT ABORT,
		file_uuid INTEGER,
		x REAL,
		y REAL,
		size REAL,
		angle REAL,
		response REAL,
		octave INT,
		CONSTRAINT references_file FOREIGN KEY (file_uuid) REFERENCES files
	)";

	let connection = open_sqlite_connection();

	let _num_rows_changed = connection
		.execute(CREATE_TABLE_FILES_STRING, params![])
		.expect("Creating database 'files' table failed");

	let _num_rows_changed = connection
		.execute(CREATE_TABLE_METADATA_STRING, params![])
		.expect("Creating database 'metadata' table failed");

	close_sqlite_connection(connection);
}

pub fn get_max_uuid() -> u64 {
	const SELECT_MAX_UUID_STRING: &str = "SELECT COALESCE(MAX(uuid), 0) FROM metadata";

	let connection = open_sqlite_connection();

	let new_uuid = connection
		.query_row(SELECT_MAX_UUID_STRING, params![], |row| row.get(0))
		.expect("Getting a max uuid from database table 'metadata' failed");

	close_sqlite_connection(connection);
	return new_uuid;
}

type FrameMetaDataPair = (FrameInfo, Vec<(u64, KeyPoint)>);
pub fn insert_meta_data_pair_vec_to_database(list: Vec<FrameMetaDataPair>) {
	let connection = open_sqlite_connection();
	connection
		.execute_batch("BEGIN")
		.expect("Starting transaction failed.");

	let mut get_max_file_uuid_statement = connection
		.prepare("SELECT COALESCE(MAX(file_uuid), 0) FROM files;")
		.expect("Preparing statement to get max file_uuid failed.");
	let mut insert_into_files_statement = connection
		.prepare(
			"INSERT INTO files 
			(file_uuid, md5, file_ext, frame_id)
			VALUES (?1, ?2, ?3, ?4);",
		)
		.expect("Preparing statement to insert into database table 'files' failed.");
	let mut insert_into_metadata_statement = connection
		.prepare(
			"INSERT INTO metadata
			(uuid, file_uuid, x, y, size, angle, response, octave)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
		)
		.expect("Preparing statement to insert into database table 'metadata' failed.");

	let total = list.len();
	for (counter, (frame, metadata_vec)) in list.into_iter().enumerate() {
		let file_uuid = {
			let result = insert_and_get_file_uuid(
				&mut get_max_file_uuid_statement,
				&mut insert_into_files_statement,
				frame,
			);
			result.expect("Inserting into database table 'files' failed. It was likely due to a md5 duplication. VP-Tree database is likely corrupted/tainted if things were added with threads")
			//	if result.is_err() {
			//		continue
			//	} else {
			//		result.unwrap()
			//	}
		};

		for (uuid, keypoint) in metadata_vec {
			insert_metadata_info_into_database(
				&mut insert_into_metadata_statement,
				(uuid, file_uuid, keypoint),
			);
		}

		if counter % 1000 == 0 {
			println!("Sqlite3 insert {} out of {}", counter, total);
		}
	}

	std::mem::drop(get_max_file_uuid_statement);
	std::mem::drop(insert_into_files_statement);
	std::mem::drop(insert_into_metadata_statement);
	connection
		.execute_batch("COMMIT;")
		.expect("Committing transaction failed.");
	close_sqlite_connection(connection);
}

fn insert_and_get_file_uuid(
	file_uuid_statement: &mut Statement,
	insert_statement: &mut Statement,
	frame: FrameInfo,
) -> Result<u64, String> {
	let max_file_uuid: u64 = file_uuid_statement
		.query_row(params![], |row| row.get(0))
		.expect("Getting a max file_uuid from database table 'files' failed");
	let max_file_uuid = max_file_uuid + 1;

	let file_insert = insert_statement.execute(params![
		max_file_uuid,
		frame.copy_md5(),
		frame.copy_ext(),
		frame.get_id()
	]);

	if file_insert.is_err() {
		return Err(String::from(
			"Inserting into database table 'files' failed. Likely md5 repeat, should skip",
		));
	} else {
		return Ok(max_file_uuid);
	}
}

fn insert_metadata_info_into_database(statement: &mut Statement, info: (u64, u64, KeyPoint)) {
	statement
		.execute(params![
			info.0,
			info.1,
			info.2.pt.x,
			info.2.pt.y,
			info.2.size,
			info.2.angle,
			info.2.response,
			info.2.octave
		])
		.expect("Failed inserting values into database");
}

#[allow(dead_code)]
pub struct KeypointMetadata {
	// My data
	uuid: u64,
	pub md5: String,
	file_ext: String,

	// Keypoint data
	x: f32,
	y: f32,
	size: f32,
	angle: f32,
	response: f32,
	octave: u8,
}

pub fn find_metadata_from_uuid(uuid_to_find: u64) -> KeypointMetadata {
	let connection = open_sqlite_connection();

	let matching_row = connection
		.query_row(
			"SELECT * FROM metadata INNER JOIN files USING (file_uuid) WHERE uuid = ?1",
			params![uuid_to_find],
			|row| row_to_keypoint_metadata(row),
		)
		.expect("Getting a max file_uuid from database table 'files' failed");

	close_sqlite_connection(connection);
	return matching_row;

	fn row_to_keypoint_metadata(row: &rusqlite::Row) -> Result<KeypointMetadata, rusqlite::Error> {
		return Ok(KeypointMetadata {
			uuid: row.get("uuid")?,
			md5: row.get("md5")?,
			file_ext: row.get("file_ext")?,

			x: row.get("x")?,
			y: row.get("y")?,
			size: row.get("size")?,
			angle: row.get("angle")?,
			response: row.get("response")?,
			octave: row.get("octave")?,
		});
	}
}

fn open_sqlite_connection() -> Connection {
	return Connection::open_with_flags(
		crate::constants::SQLITE_DATABASE_PATH,
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
	)
	.expect("Creating database file failed");
}

fn close_sqlite_connection(connection: Connection) {
	connection.close().expect("Closing database file failed");
}
