use directories;
use crate::frecent_paths::PathFrecency;
use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};
pub struct Fasd;

pub struct ImportStats {
    pub items_considered: u64,
    pub items_visited: u64,
}

impl Fasd {
    pub fn import(db: &mut PathFrecency) -> Result<ImportStats, String> {
        let fasd_env = env::var("_FASD_DATA");
        let fasd_data = match fasd_env {
            // For proper compatibility, this should use 'shellexpand' or a similar crate.
            Ok(dir) => PathBuf::from(dir),
            Err(_) => {
                let user_dirs =
                    directories::UserDirs::new().ok_or_else(|| "could not get home dir")?;
                let home = user_dirs.home_dir();
                home.join(".fasd")
            }
        };

        let f = fs::File::open(&fasd_data)
            .map_err(|e| format!("could not open {:?} for import: {}", &fasd_data, e))?;

        let mut stats = ImportStats {
            items_visited: 0,
            items_considered: 0,
        };

        for line in BufReader::new(f).lines() {
            let line = line.map_err(|e| format!("error reading {:?}: {}", &fasd_data, e))?;
            let data = match line.splitn(2, "|").next() {
                None => {
                    warn!("Incorrectly formatted fasd data line: {}", line);
                    continue;
                }
                Some(d) => d,
            };
            stats.items_considered += 1;

            if Path::new(&data).is_dir() {
                debug!("visiting: {}", data);
                db.visit(data.to_string());
                stats.items_visited += 1;
            }
        }

        Ok(stats)
    }
}
