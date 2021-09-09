# VP-Tree-Database
A database for image retrieval using [ORB](https://docs.opencv.org/3.4/d1/d89/tutorial_py_orb.html) and a [VP Tree](https://fribbels.github.io/vptree/writeup).

## How to Build
* Have OpenCV installed and functioning (https://lib.rs/crates/opencv)
* Have Sqlite3 installed and functioning (https://lib.rs/crates/rusqlite)
* Run `cargo build --release`

## How to use
* Add an image `./feature_database -a /path/to/image`
* Query an image `./feature_database -f /path/to/image`
* Rank results of a query `./feature_database -f /path/to/image | sort | uniq -c | sort -n -k1`

## How to improve
* Create a folder called `database` and mount it as a [ramdisk](https://www.jamescoyle.net/how-to/943-create-a-ram-disk-in-linux). (Warning, data will be lost on reboot or unmount)

## How it works
When adding an image, it is run through [ORB](https://docs.opencv.org/3.4/d1/d89/tutorial_py_orb.html) and ~500 features with metadata are extracted. Each metadata-feature pair are assigned a unique id (mislabeled as uuid) and the metadata is saved to a sqlite3 database.

The features obtained with ORB are a 32 dimensional vector and because of this necessitate a unique storage method. There are [a plethora of trees](https://en.wikipedia.org/wiki/Template:CS_trees) to choose from, but after some testing in python, the VP-Tree performed the best and seemed the easiest to build.
