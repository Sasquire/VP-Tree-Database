pub const APP_NAME: &str = "VP-Tree-Database";
pub const VERSION: &str = "0.00001";
pub const CONTACT_INFO: &str = "Sasquire (idem on e621.net)";
pub const ABOUT: &str = "A database using a vantage-point tree for reverse-image-searches";
pub const LONG_ABOUT: &str = "
~ is a image-retrevial system.
It uses ORB to extract featues from images and store the descriptors in a
VP-Tree for later querying. ~ was written because none of the options I found
while creating it could handle a large amount of data. To combat this issue ~
tries to have many smaller files so less RAM will be used. While performance
is lost to disk I/O, the ~ is able to add and query a database that is larger
than system memory.

To add images use the -a option and to query images use the -f option.
";

pub const DATABASE_FOLDER_PATH: &str = "./database/";
pub const SQLITE_DATABASE_PATH: &str = "./database/metadata.sqlite3";
pub const THREADED_INSERT: bool = false;

pub const MAX_LEAF_NODE_SIZE: u64 = 4096 * 2;
pub const MAX_FILE_NODE_DEPTH: usize = 8;
pub const FILE_NODE_MEMORY_SAVER: bool = false;

pub const FEATURE_DESCRIPTION_LENGTH: usize = 32;
// 694960 is the default radius because it is equal to
// 32 * Average(SUM_0^255 x^2)
// which is the expected value of the distance of a random node
// to the origin. Experimental results give results that are
// close to this value, so I think it is correct.
pub const AVERAGE_EDGE_FEATURE_DISTANCE: u32 = 694_960;

// Keys used to tell what direction a Node Path is going down
pub const NEAR_KEY: u8 = b'n';
pub const FAR_KEY: u8 = b'a';
pub const FILE_KEY: u8 = b'l';
pub const UNUSED_KEY: u8 = b'u';

// Signature like a file signature https://en.wikipedia.org/wiki/List_of_file_signatures
// pub const SIGNATURE_LENGTH: usize = 4;
pub const SIGNATURE_RANGE: std::ops::Range<usize> = 0..4;
pub const LEAF_NODE_SIGNATURE: &str = "leaf";
pub const INTERNAL_NODE_SIGNATURE: &str = "intr";
pub const FILE_NODE_SIGNATURE: &str = "file";

pub const DEFAULT_K: usize = 100;
pub const MAX_K_VALUE: usize = 1000;
pub const THREADED_SEARCH: bool = false;
