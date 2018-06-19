use std::{
    sync::Arc,
    collections::HashMap,
    path,
    io::{
        self,
        Read,
    },
    fs::{
        read_dir,
        File,
    },
};
use helpers::{
    Path,
    PathBuf,
};
use lexeme_scanner::rules::word::word;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    pub name: String,
    pub text: String,
}

impl Text {
    pub fn new<A: ToString, B: ToString>(name: A, text: B) -> Self {
        Text { name: name.to_string(), text: text.to_string() }
    }
}

pub trait TextSource {
    fn get_text(&self, path: Path) -> Option<Arc<Text>>;
}

#[derive(Clone)]
pub struct HashMapSource {
    map: HashMap<PathBuf, Arc<Text>>,
}

fn extract_file_name(path: &path::Path) -> io::Result<&str> {
    let filename = path.file_name()
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Could not extract filename from {:?}.", path),
        ))?;
    filename.to_str()
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Could not convert filename {:?} into unicode charset.", filename),
        ))
}

fn make_module_name(filename: &str) -> io::Result<Path> {
    let (_, length) = word(filename.as_bytes())
        .map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Could not convert filename {:?} into module name.", filename),
        ))?;
    Ok(Path::new(&filename[..length], "::"))
}

pub const N_LANG_FILE_EXTENSIONS: &'static str = "n";

impl HashMapSource {
    pub fn new() -> Self {
        HashMapSource {
            map: HashMap::new(),
        }
    }
    pub fn simple_insert(&mut self, path: Path, name: &str, text: &str) {
        self.map.insert(
            PathBuf::from_path(path),
            Arc::new(Text {
                name: name.to_string(),
                text: text.to_string(),
            }),
        );
    }
    pub fn insert(&mut self, path: Path, name: String, text: String) {
        self.map.insert(
            PathBuf::from_path(path),
            Arc::new(Text {
                name,
                text,
            }),
        );
    }
    pub fn for_dir(path: &path::Path) -> io::Result<HashMapSource> {
        let mut result = HashMapSource::new();
        for entry in read_dir(path)? {
            let path = entry?.path();
            if !path.is_file() { continue; }
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some(ext) => if ext != N_LANG_FILE_EXTENSIONS { continue; }
                    None => continue,
                },
                None => continue,
            }
            let file_name = extract_file_name(&path)?;
            let module_name = make_module_name(file_name)?;
            let mut file = File::open(&path)?;
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            result.insert(module_name, file_name.to_string(), text);
        }
        Ok(result)
    }
}

impl TextSource for HashMapSource {
    fn get_text(&self, path: Path) -> Option<Arc<Text>> {
        for (text_path, text) in self.map.iter() {
            if text_path.as_path() == path {
                return Some(text.clone());
            }
        }
        None
    }
}
