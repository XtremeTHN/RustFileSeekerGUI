use walkdir::WalkDir;
use std::path::PathBuf;

use std::collections::HashMap;


#[derive(Debug)]
pub struct Finder {
    path: String,
    exts: Option<Vec<String>>,
    files_by_ext: Option<HashMap<String, Vec<PathBuf>>>,
    all: Option<Vec<PathBuf>>
}

impl Finder {
    pub fn new(path: String, extension_filter: String) -> Self {
        let exts_vec: Vec<&str> = extension_filter.split("|").collect();
        let vector_string: Vec<String> = exts_vec.iter().map(|&s| String::from(s)).collect();

        Finder { path: path, exts: Some(vector_string), files_by_ext: Some(HashMap::new()), all: Some(vec![])}
    }

    pub fn find(&mut self) -> Result<HashMap<String, Vec<PathBuf>>, ()> {
        let mut files_by_ext: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for entry in WalkDir::new(&self.path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.starts_with(".") {
                continue;
            }
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_string();
                if let Some(exts) = &self.exts {
                    if exts.iter().any(|e| e == &ext) {
                        files_by_ext.entry(ext).or_insert(vec![]).push(path.to_path_buf());
                        if let Some(all) = self.all.as_mut() {
                            all.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
        if files_by_ext.is_empty() {
            return Err(());
        }
        Ok(files_by_ext)
    }

    pub fn get_all(&self) -> Vec<PathBuf> {
        // if let Some(vec) = self.all.clone() {
        //     vec.into_iter()
        //     .map(|pb| pb.into_os_string().into_string().unwrap())
        //     .collect()
        // } else {
        //     vec![]
        // }
        self.all.clone().unwrap()
    }
    
    pub fn filter(&mut self, keyword: &str) -> Vec<PathBuf> {
        let mut matches = vec![];
        if let Some(files) = self.all.as_ref() {
            for file in files {
                if let Some(file_name) = file.file_name() {
                    if file_name.to_string_lossy().contains(keyword) {
                        matches.push(file.clone());
                    }
                }
            }
        }
        return matches;
    }
}