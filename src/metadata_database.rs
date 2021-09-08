// TODO this whole file is ugly and needs to be cleaned up

use crate::frame_info::FrameInfo;

use opencv::core::KeyPoint;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;

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

pub fn insert_metadata_vec_into_database(metadata_vec: Vec<(u64, KeyPoint)>, frame: FrameInfo) {
	let file_uuid = add_and_get_file_uuid(frame);

	let connection = open_sqlite_connection();
	for (uuid, keypoint) in metadata_vec {
		insert_metadata_to_database(&connection, &file_uuid, uuid, keypoint);
	}
	close_sqlite_connection(connection);

	fn insert_metadata_to_database(
		database: &Connection,
		file_uuid: &u64,
		uuid: u64,
		metadata: KeyPoint,
	) {
		const INSERT_KEYPOINT_TO_METADATA_TABLE_STRING: &str = "INSERT INTO metadata (
			uuid,
			file_uuid,
			x,
			y,
			size,
			angle,
			response,
			octave
		) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);";
		let params = params![
			uuid,
			file_uuid,
			metadata.pt.x,
			metadata.pt.y,
			metadata.size,
			metadata.angle,
			metadata.response,
			metadata.octave
		];

		database
			.execute(INSERT_KEYPOINT_TO_METADATA_TABLE_STRING, params)
			.expect("Failed inserting values into database");
	}
}

fn add_and_get_file_uuid(frame: FrameInfo) -> u64 {
	let connection = open_sqlite_connection();

	let max_file_uuid = get_max_file_uuid(&connection) + 1;
	insert_new_file(&connection, max_file_uuid, frame);
	close_sqlite_connection(connection);
	return max_file_uuid;

	fn get_max_file_uuid(connection: &Connection) -> u64 {
		const SELECT_MAX_FILE_UUID_STRING: &str = "SELECT COALESCE(MAX(file_uuid), 0) FROM files";

		return connection
			.query_row(SELECT_MAX_FILE_UUID_STRING, params![], |row| row.get(0))
			.expect("Getting a max file_uuid from database table 'files' failed");
	}

	fn insert_new_file(connection: &Connection, file_uuid: u64, frame: FrameInfo) {
		const INSERT_FILE_TO_FILES_TABLE: &str = "INSERT INTO files (
			file_uuid,
			md5,
			file_ext,
			frame_id
		) VALUES (?1, ?2, ?3, ?4);";

		let params = params![
			file_uuid,
			frame.copy_md5(),
			frame.copy_ext(),
			frame.get_id()
		];

		connection
			.execute(INSERT_FILE_TO_FILES_TABLE, params)
			.expect("Inserting into database table 'files' failed. Likely file already exists.");
	}
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
