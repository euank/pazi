use crate::frecent_paths::PathFrecencyDiff;
use snailquote;
use std::char;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str;
use tempfile::Builder;
use which;

// edit opens up EDITOR with the given input matches for the user to edit. It returns a 'diff' of
// what has changed.
pub fn edit(data: &Vec<(String, f64)>) -> Result<PathFrecencyDiff, String> {
    let mut editor = env::var("PAZI_EDITOR")
        .or_else(|_| env::var("EDITOR"))
        .or_else(|_| env::var("VISUAL"))
        .map_err(|_| "please set PAZI_EDITOR or EDITOR")
        .and_then(|ed| {
            // support 'EDITOR=bin args' by splitting out possible args
            // note, this unfortunately means editors with spaces in their paths won't work.
            // This matches systemd's behavior as best I can tell:
            // https://github.com/systemd/systemd/blob/d32d473d66caafb4e448fa5fc056589b7763c478/src/systemctl/systemctl.c#L6795-L6803
            let mut parts = ed.split(char::is_whitespace);

            let edpart = parts.next();
            let rest = parts.map(|s| s.to_owned()).collect();
            match edpart {
                Some(s) => Ok((PathBuf::from(s), rest)),
                None => Err("empty editor"),
            }
        })
        .or_else(|_| {
            for bin in vec!["editor", "nano", "vim", "vi"] {
                match which::which(bin) {
                    Ok(ed) => {
                        return Ok((ed, vec![]));
                    }
                    Err(_) => (),
                }
            }
            Err("could not find editor in path")
        })?;

    let mut tmpf = Builder::new()
        .prefix("pazi_edit")
        .tempfile()
        .map_err(|e| format!("error creating tempfile: {}", e))?;

    let serialized_data = serialize(data);
    tmpf.write_all(serialized_data.as_bytes())
        .map_err(|e| format!("could not write data to tempfile: {}", e))?;

    debug!("created tmpfile at: {}", tmpf.path().to_str().unwrap());
    editor.1.push(tmpf.path().to_str().unwrap().to_string());
    let mut cmd = Command::new(editor.0);
    cmd.args(editor.1);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("error spawning editor: {}", e))?;

    let exit = child
        .wait()
        .map_err(|e| format!("error waiting for editor: {}", e))?;

    if !exit.success() {
        return Err(format!("editor exited non-zero: {}", exit.code().unwrap()))?;
    }
    tmpf.seek(SeekFrom::Start(0))
        .map_err(|e| format!("could not seek in tempfile: {}", e))?;
    let mut new_contents = String::new();
    tmpf.read_to_string(&mut new_contents)
        .map_err(|e| format!("error reading file: {}", e))?;

    if new_contents.trim() == serialized_data.trim() {
        debug!("identical data read; shortcutting out");
        return Ok(PathFrecencyDiff::new(Vec::new(), Vec::new()));
    }

    let mut new_map = deserialize(&new_contents)?;

    let mut removals = Vec::new();
    let mut additions = Vec::new();

    // Find all items in the input set and see if they've changed
    for item in data {
        debug!("edit: processing {:?}", item);
        match new_map.remove(&item.0) {
            Some(w) => {
                if item.1 != w {
                    debug!("edit: update {:?}", item.0);
                    // weight edited, aka remove + add
                    removals.push(item.0.clone());
                    additions.push((item.0.clone(), w));
                }
                // otherwise, no diff
            }
            None => {
                debug!("edit: removal {:?}", item.0);
                removals.push(item.0.clone());
            }
        }
    }
    // anything that was in the input has been removed from our new_map now, so anything
    // remaining is an addition to the database
    for (k, v) in new_map {
        debug!("edit: add {:?}", k);
        additions.push((k, v));
    }

    Ok(PathFrecencyDiff::new(additions, removals))
}

pub fn serialize(matches: &Vec<(String, f64)>) -> String {
    format!(
        r#"# Edit your frecency fearlessly!
#
# Lines starting with '#' are comments. sh-esque quoting and escapes may be used in paths.
# Columns are whitespace separated. The first column is the current score, the second is the path.
# Any changes saved here will be applied back to your frecency database immediately.
{}"#,
        matches
            .iter()
            .map(|(s, w)| format!("{}\t{}", w, snailquote::escape(s)))
            .collect::<Vec<String>>()
            .join("\n")
    )
}

pub fn deserialize(s: &str) -> Result<HashMap<String, f64>, String> {
    let mut res = HashMap::new();
    for mut line in s.lines() {
        line = line.trim();

        if line.len() == 0 || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();

        if parts.len() != 2 {
            return Err(format!(
                "line '{}' did not have whitespace to split on",
                line
            ));
        }

        let path = snailquote::unescape(parts[1])
            .map_err(|e| format!("error unescaping edited path: {}: {}", parts[1], e))?;
        let w = parts[0]
            .parse::<f64>()
            .map_err(|e| format!("could not parse {} as float: {}", parts[1], e))?;

        res.insert(path, w);
    }

    Ok(res)
}
