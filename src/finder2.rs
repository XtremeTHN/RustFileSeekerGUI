use walkdir::WalkDir;
use std::{path::PathBuf, hash::Hash, borrow::Borrow};
use serde::Deserialize;
use std::collections::HashMap;
use log::{info, error, warn};

pub struct Finder {
    path: PathBuf,
    exts: Vec<String>,
    files_by_ext: HashMap<String, Vec<PathBuf>>,
    all: Vec<PathBuf>,
}

impl Finder {
    pub fn new(path: String, extension_filter: String) -> Self {

        let exts_vec: Vec<&str> = extension_filter.split("|").collect();
        let vector_string: Vec<String> = exts_vec.iter().map(|&s| String::from(s)).collect();
        
        Finder { path: PathBuf::from(path), exts: vector_string, files_by_ext: HashMap::new(), all: vec![]}
    }

    pub fn find(&mut self) -> Result<HashMap<String, Vec<PathBuf>>, ()> {
        let mut archivos_por_extension: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for entrada in WalkDir::new(self.path.clone()).into_iter().filter_map(Result::ok) {
            if entrada.file_type().is_file() {
                if let Some(extension) = entrada.path().extension().and_then(|ext| ext.to_str()) {
                    if self.exts.contains(&extension.to_string()) {
                        let clave = extension.to_string();
                        let valor = entrada.path().to_path_buf();
    
                        archivos_por_extension
                            .entry(clave)
                            .or_insert(Vec::new())
                            .push(valor);
                    }
                }
            }
        }
        info!("Listo");
        Ok(archivos_por_extension)
    }
    
    pub fn filter(&mut self, keyword: &str) -> Vec<PathBuf> {
        let mut matches = vec![];
        for file in &self.all {
            if let Some(file_name) = file.file_name() {
                if file_name.to_string_lossy().contains(keyword) {
                    matches.push(file.clone());
                }
            }
        }
        return matches;
    }
}