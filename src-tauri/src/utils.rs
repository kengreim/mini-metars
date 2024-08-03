use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn get_or_create_path(path: &Path) -> Option<PathBuf> {
    match path.try_exists() {
        Ok(true) => Some(path.to_path_buf()),
        Ok(false) => create_dir_all(path).map_or(None, |()| Some(path.to_path_buf())),
        _ => None,
    }
}

pub fn deserialize_from_file<T: DeserializeOwned>(path: &Path) -> Result<T, anyhow::Error>
where
    T:,
{
    let file = File::open(path)?;
    let de = serde_json::from_reader(BufReader::new(file))?;
    Ok(de)
}

pub fn serialize_to_file<T: Serialize>(path: &Path, data: &T) -> Result<(), anyhow::Error> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), data)?;
    Ok(())
}
