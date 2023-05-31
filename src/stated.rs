use adw::prelude::*;
use log::{debug, error};
use std::{time::SystemTime, path::PathBuf};
use chrono::{DateTime, Local};
use std::fs;
use gtk;

use crate::setup::YamlConfiguration;

fn convert_bytes_to_human_readable(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }

    let exp = (bytes as f64).log2() as i32 / 10;
    let unit = ['K', 'M', 'G', 'T', 'P', 'E'][exp as usize - 1];

    return format!("{:.1} {}B", bytes as f64 / 1024.0_f64.powi(exp * 10), unit);
}

pub struct Stated {
    liststore: gtk::ListStore,
    configs: YamlConfiguration,
}

impl Stated {
    pub fn new(configs: YamlConfiguration) -> Stated {
        Stated {liststore: gtk::ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]), configs: configs }
    }

    pub fn get_liststore(&self) -> gtk::ListStore{
        return self.liststore.clone();
    }

    pub fn insert_into(&self, index: u32, data: String) -> gtk::TreeIter {
        self.liststore.insert_with_values(
            None,
            &[
                (index, &data),
            ]
        )
    }

    pub fn stat_and_insert(&self, files: Vec<PathBuf>) -> Result<(), String> {
        debug!("Getting information of {} files", files.len());
        for (_, file) in files.iter().enumerate() {
            let metadata = fs::metadata(file.as_path().to_string_lossy().to_string());

            if let Err(err) = metadata {
                if !self.configs.general.skip_metadata_errors {
                    error!("Error while trying to get metadata of the file {}. Error message: {}", file.as_path().to_string_lossy(), err);
                    return Err(format!("Cannot get information of one file {}", file.as_path().to_string_lossy()));
                } else {
                    return Ok(());
                }
            }

            let metadata = metadata.unwrap();

            let modified_time = metadata.modified().unwrap();
            let system_time = SystemTime::UNIX_EPOCH + modified_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let datetime: DateTime<Local> = DateTime::from(system_time);
            
            let date_f = datetime.to_string();
            let size_f = convert_bytes_to_human_readable(metadata.len());
            let ext_f = file.extension().unwrap().to_string_lossy().to_string();
            let name_f = file.file_name().unwrap().to_string_lossy().to_string();
            let path_f = file.as_path().to_string_lossy().to_string();
            
            self.liststore.insert_with_values(None, &[
                (0, &name_f.to_value()),
                (1, &ext_f.to_value()),
                (2, &path_f.to_value()),
                (3, &size_f.to_value()),
                (4, &date_f.to_value()),
            ]);
            
        }
        Ok(())
    }
}
