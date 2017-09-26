// frecent_paths is a specialization of frecency that understands the semantics of stored paths.
// It does things like the messyness of checking for a directory's existence and such.

use frecency::Frecency;
use std::path::{Path, PathBuf};
use std::fs;
use serde::Serialize;
use serde;
use libc;
use rmp_serde;

pub struct PathFrecency {
    frecency: Frecency<String>,
    path: PathBuf,
}


impl PathFrecency {
    // load loads or, if it doesn't exist, creates a path frecency db at a given location
    pub fn load(path: &Path) -> Self {
        let frecency_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        let metadata = frecency_file.metadata().unwrap();
        // remember 500 entries total
        let frecency = if metadata.len() > 0 {
            // existing file, unmarshal that sucker
            let mut de = rmp_serde::Deserializer::from_read(frecency_file);
            serde::Deserialize::deserialize(&mut de).unwrap()
        } else {
            Frecency::<String>::new(500)
        };

        PathFrecency {
            frecency: frecency,
            path: path.to_path_buf(),
        }
    }

    pub fn visit(&mut self, dir: String) {
        self.frecency.visit(dir);
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        // Transform frecency path into a temporary path for atomic move
        let my_pid = unsafe { libc::getpid() };
        if my_pid == 0 {
            return Err("could not get pid".to_string());
        }

        let fname = self.path
            .file_name()
            .ok_or("path did not have file component".to_string())?;

        let tmpfile_name = format!(".{}.{}", fname.to_string_lossy(), my_pid);
        let tmpfile_dir = self.path.parent().ok_or("unable to get parent".to_string())?;
        let tmpfile_path = tmpfile_dir.join(tmpfile_name);

        let tmpfile =
            fs::File::create(&tmpfile_path).map_err(|_| "could not create tempfile".to_string())?;

        self.frecency
            .serialize(&mut rmp_serde::Serializer::new(tmpfile))
            .map_err(|_| "could not create tmpfile".to_string())?;
        fs::rename(tmpfile_path, &self.path).map_err(|e| {
            format!("could not atomically rename: {}", e).to_string()
        })
    }

    pub fn items(&self) -> Vec<&String> {
        self.frecency.items()
    }

    pub fn items_with_frecency(&self) -> Vec<(&String, &f64)> {
        self.frecency.items_with_frecency()
    }
}
