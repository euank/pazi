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

        let fname = self.path.file_name().ok_or(
            "path did not have file component"
                .to_string(),
        )?;

        let tmpfile_name = format!(".{}.{}", fname.to_string_lossy(), my_pid);
        let tmpfile_dir = self.path.parent().ok_or("unable to get parent".to_string())?;
        let tmpfile_path = tmpfile_dir.join(tmpfile_name);

        let tmpfile = fs::File::create(&tmpfile_path).map_err(|_| {
            "could not create tempfile".to_string()
        })?;

        self.frecency
            .serialize(&mut rmp_serde::Serializer::new(tmpfile))
            .map_err(|_| "could not create tmpfile".to_string())?;
        fs::rename(tmpfile_path, &self.path).map_err(|e| {
            format!("could not atomically rename: {}", e).to_string()
        })
    }

    pub fn items_with_normalized_frecency(&self) -> Vec<(&String, f64)> {
        let items = self.frecency.normalized_frecency();
        if items.len() == 0 {
            return items;
        }
        let min = items[items.len() - 1].1;
        let max = items[0].1;

        items
            .into_iter()
            .map(|(s, v)| {
                let normalized = (v - min) / max;
                (s, normalized)
            })
            .collect()
    }

    pub fn best_directory_match(&self, filter: &str) -> Option<&String> {
        // 'best directory' is a tricky concept, as is 'match.
        //
        // There's a continuum from "exact string match" to "no characters in common", and we
        // have to try and approximate what a human expects to figure out the weight and cutoff
        // within that continuum.
        //
        // The following assumptions are what I started with:
        // 1) Exact matches should always be jumped to with no questions asked. Exact matches are
        //    rare. Substring matches are permissible and expected.
        // 2) Components should be deconstructed from frecency database items for matching; people
        //    think in components. For example, an entry of "/home/user/dev" will be thought about
        //    by a user as the three distinct components "home", "user", and "dev", so we can
        //    better match their expectations by matching individual components.
        // 3) Component matches should be weighted based on how "deep" / "far right" the matched
        //    component is. That is to say, the query "foo" should be weighted more highly for
        //    "/home/user/project/foo" than for "/home/user/foo/stuff", even if the latter is
        //    higher in the frecency index.
        // 4) Case and punctuation in the target are liable to not be present in the query.
        // 5) If the query contains a component separator, the user likely wants each side of it to
        //    be fuzzy. That is to say: "z dev/tool" likely wishes to do a fuzzy match on the
        //    strings "dev" and "tool" on adjacent components, leading to results like
        //    "dev/my-tool" being possible.
        // 6) Levenshtein distance may be fallen back upon for real "fuzzyness", but should be
        //    weighted carefully low; sometimes it is better to force a user to make a new query
        //    than to make too strange of a shot in the dark.

        let ci_em = case_insensitive_matcher(&ExactMatcher {});
        let pc_ci_em = PathComponentMatcher { base: &ci_em };
        let ci_sm = case_insensitive_matcher(&SubstringMatcher {});
        let pc_ci_sm = PathComponentMatcher { base: &ci_sm };
        let matchers: Vec<&Matcher> = vec![
            &ExactMatcher {},
            &ci_em,
            &PathComponentMatcher { base: &ExactMatcher {} },
            &PathComponentMatcher { base: &SubstringMatcher {} },
            &pc_ci_em,
            &SubstringMatcher {},
            &ci_sm,
            &pc_ci_sm,
        ];

        let best = self.items_with_normalized_frecency()
            .iter().flat_map(|item| {
                matchers.iter().filter_map(move |m| {
                    match m.matches(item.0, filter) {
                        Some(v) => Some((item.0, v * 0.6 +  item.1 * 0.4)),
                        None => None,
                    }
                })
            }).max_by(|lhs, rhs| {
                // unwrap for NaN which shouldn't happen
                lhs.1.partial_cmp(&rhs.1).unwrap()
            });


        best.and_then(|el| {
            if el.1 > (0.5 * 0.5) {
                Some(el.0)
            } else {
                debug!("discarding {} with score {}", el.0, el.1);
                None
            }
        })
    }
}

trait Matcher {
    fn matches(&self, input: &str, search: &str) -> Option<f64>;
}

struct ExactMatcher {}
impl Matcher for ExactMatcher {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        if input == search {
            return Some(1.0);
        }
        None
    }
}

struct SubstringMatcher {}
impl Matcher for SubstringMatcher {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        let res = input.find(search);
        match res {
            None => None,
            Some(offset) => {
                let base = if offset == 0 {
                    // If the match is at the very beginning, consider it a better match
                    1.0
                } else {
                    0.8
                };
                Some(base * search.len() as f64 / input.len() as f64)
            }
        }
    }
}

struct PathComponentMatcher<'a> {
    base: &'a Matcher,
}
impl<'a> Matcher for PathComponentMatcher<'a> {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        let p = Path::new(input);
        let components = p.components();
        let num_components = p.components().count();
        // Reduce the weight of components the further from the right they are
        // I've arbitrarily chosen to linearly attenuate them
        let mut weight = 0.9;
        let weight_step = (weight - 0.2) / num_components as f64;
        let mut res = None;
        for component in components.rev() {
            let s = match component.as_os_str().to_str() {
                Some(s) => s,
                None => {
                    continue;
                }
            };
            match self.base.matches(s, search) {
                Some(v) => {
                    let attv = v * weight;
                    res = match res {
                        None => Some(attv),
                        Some(existing) => {
                            if attv > existing {
                                Some(attv)
                            } else {
                                Some(existing)
                            }
                        }
                    }
                }
                None => {}
            }
            weight -= weight_step;
        }
        res
    }
}


struct TransformedMatcher<'a> {
    transformation: fn(input: &str) -> String,
    matcher: &'a Matcher,
    attenuation: f64,
}

fn case_insensitive_matcher(base: &Matcher) -> TransformedMatcher {
    fn transformer(input: &str) -> String {
        input.to_owned().to_lowercase()
    }
    TransformedMatcher {
        transformation: transformer,
        matcher: base,
        attenuation: 0.7,
    }
}


impl<'a> Matcher for TransformedMatcher<'a> {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        match self.matcher.matches(&(self.transformation)(input), search) {
            Some(f) => Some(f * self.attenuation),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_match_in_order(match_results: Vec<Option<f64>>) {
        for i in 0..(match_results.len() - 1) {
            match (match_results[i], match_results[i + 1]) {
                (Some(lhs), Some(rhs)) => {
                    assert!(rhs > lhs);
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_exact_matcher() {
        let m = ExactMatcher {};
        assert_eq!(m.matches("foo", "foo"), Some(1.0));
        assert_eq!(m.matches("i pity", "the fool"), None);
        assert_eq!(m.matches("FOO", "foo"), None);
    }

    #[test]
    fn test_case_insensitive_matcher() {
        let em = ExactMatcher {};
        let ci = case_insensitive_matcher(&em);

        assert_eq!(ci.matches("foo", "foo"), Some(0.7));
        assert_eq!(ci.matches("i pity", "the fool"), None);
        assert_eq!(ci.matches("FOO", "foo"), Some(0.7));
    }

    #[test]
    fn test_path_component_matcher() {
        let em = ExactMatcher {};
        let pc = PathComponentMatcher { base: &em };

        assert_match_in_order(vec![
            pc.matches("/foo/bar", "foo"),
            pc.matches("/foo", "foo"),
        ]);
        assert_eq!(pc.matches("/foo", "foo"), pc.matches("/asdf/foo", "foo"));
        assert_eq!(pc.matches("/foo/bar", "ar"), None);
    }

    #[test]
    fn test_substring_matcher() {
        let sm = SubstringMatcher {};

        assert_match_in_order(vec![
            sm.matches("foo", "f"),
            sm.matches("foo", "fo"),
            sm.matches("foo", "foo"),
        ]);
    }
}
