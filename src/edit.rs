use std::char;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str;

use anyhow::{anyhow, bail, Context, Result};
use tempfile::Builder;

use super::frecent_paths::PathFrecencyDiff;

// edit opens up EDITOR with the given input matches for the user to edit. It returns a 'diff' of
// what has changed.
pub fn edit(data: &[(String, f64)]) -> Result<PathFrecencyDiff> {
    let mut editor = env::var("PAZI_EDITOR")
        .or_else(|_| env::var("EDITOR"))
        .or_else(|_| env::var("VISUAL"))
        .map_err(|_| anyhow!("please set PAZI_EDITOR or EDITOR"))
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
                None => Err(anyhow!("empty editor")),
            }
        })
        .or_else(|_| {
            for bin in &["editor", "nano", "vim", "vi"] {
                if let Ok(ed) = which::which(bin) {
                        return Ok((ed, vec![]));
                }
            }
            Err(anyhow!("could not find editor in path"))
        })?;

    let mut tmpf = Builder::new()
        .prefix("pazi_edit")
        .tempfile()
        .with_context(|| "error creating tempfile")?;

    let serialized_data = serialize(data);
    tmpf.write_all(serialized_data.as_bytes())
        .with_context(|| "could not write data to tempfile")?;

    debug!("created tmpfile at: {}", tmpf.path().to_str().unwrap());
    editor.1.push(tmpf.path().to_str().unwrap().to_string());
    let mut cmd = Command::new(editor.0);
    cmd.args(editor.1);

    let mut child = cmd.spawn().with_context(|| "error spawning editor")?;

    let exit = child.wait().with_context(|| "error waiting for editor")?;

    if !exit.success() {
        bail!("editor exited non-zero: {}", exit.code().unwrap());
    }
    tmpf.seek(SeekFrom::Start(0))
        .with_context(|| "could not seek in tempfile")?;
    let mut new_contents = String::new();
    tmpf.read_to_string(&mut new_contents)
        .with_context(|| "error reading tempfile")?;

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
                if (item.1 - w).abs() > f64::EPSILON {
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

pub fn serialize(matches: &[(String, f64)]) -> String {
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

pub fn deserialize(s: &str) -> Result<HashMap<String, f64>> {
    let mut res = HashMap::new();
    for mut line in s.lines() {
        line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();

        if parts.len() != 2 {
            bail!("line '{}' did not have whitespace to split on", line);
        }

        let path = snailquote::unescape(parts[1])
            .map_err(|e| anyhow!("error unescaping edited path: {}: {}", parts[1], e))?;
        let w = parts[0]
            .parse::<f64>()
            .map_err(|e| anyhow!("could not parse {} as float: {}", parts[1], e))?;

        res.insert(path, w);
    }

    Ok(res)
}
