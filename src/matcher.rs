use std::path::Path;

pub trait Matcher {
    fn matches(&self, input: &str, search: &str) -> Option<f64>;
}

pub struct ExactMatcher {}
impl Matcher for ExactMatcher {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        if input == search {
            return Some(1.0);
        }
        None
    }
}

pub struct SubstringMatcher {}
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

pub struct PathComponentMatcher<'a>(&'a Matcher);

impl<'a> PathComponentMatcher<'a> {
    pub fn new(base: &'a Matcher) -> Self {
        PathComponentMatcher(base)
    }
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
            match self.0.matches(s, search) {
                Some(v) => {
                    let attv = v * weight;
                    res = match res {
                        None => Some(attv),
                        Some(existing) => if attv > existing {
                            Some(attv)
                        } else {
                            Some(existing)
                        },
                    }
                }
                None => {}
            }
            weight -= weight_step;
        }
        res
    }
}

pub struct TransformedMatcher<'a> {
    input_transformation: fn(input: &str) -> String,
    search_transformation: fn(input: &str) -> String,
    matcher: &'a Matcher,
    attenuation: f64,
}

pub type CaseInsensitiveMatcher<'a> = TransformedMatcher<'a>;

impl<'a> CaseInsensitiveMatcher<'a> {
    pub fn new(base: &'a Matcher) -> Self {
        fn transformer(input: &str) -> String {
            input.to_owned().to_lowercase()
        }
        TransformedMatcher {
            input_transformation: transformer,
            search_transformation: transformer,
            matcher: base,
            attenuation: 0.7,
        }
    }
}

impl<'a> Matcher for TransformedMatcher<'a> {
    fn matches(&self, input: &str, search: &str) -> Option<f64> {
        match self.matcher.matches(
            &(self.input_transformation)(input),
            &(self.search_transformation)(search),
        ) {
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
        let ci = CaseInsensitiveMatcher::new(&em);

        assert_eq!(ci.matches("foo", "foo"), Some(0.7));
        assert_eq!(ci.matches("i pity", "the fool"), None);
        assert_eq!(ci.matches("FOO", "foo"), Some(0.7));
        assert_eq!(ci.matches("foo", "FOO"), Some(0.7));
        assert_eq!(ci.matches("aSdF", "AsDf"), Some(0.7));
    }

    #[test]
    fn test_path_component_matcher() {
        let em = ExactMatcher {};
        let pc = PathComponentMatcher::new(&em);

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
