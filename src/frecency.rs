use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::f64;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

const DECAY_RATE: f64 = f64::consts::LN_2 / (30. * 24. * 60. * 60.);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Frecency<T>
where
    T: Hash + Eq + Ord + Clone,
{
    // ordering is enforced on access, not on store. This is because updating an entry (visiting)
    // is a much more frequent operation than searching through items for this program.
    frecency: HashMap<T, f64>,
    max_size: usize,
}


impl<T> Frecency<T>
where
    T: Hash + Eq + Ord + Clone,
{
    pub fn new(max_size: usize) -> Self {
        Frecency {
            frecency: HashMap::new(),
            max_size: max_size,
        }
    }

    pub fn visit(&mut self, key: T) {
        self.visit_with_time(key, SystemTime::now())
    }

    // based off https://wiki.mozilla.org/User:Jesse/NewFrecency#Proposed_new_definition
    fn visit_with_time(&mut self, key: T, now: SystemTime) {
        // The only error here is if the system clock is before the unix epoch. I'm fine panicing
        // there.
        let now_secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let now_decay = now_secs as f64 * DECAY_RATE;
        match self.frecency.entry(key) {
            Entry::Occupied(mut e) => {
                let frecency = e.get_mut();
                *frecency = ((*frecency - now_decay).exp() + 1f64).ln() + now_decay;
            }
            Entry::Vacant(e) => {
                e.insert(now_decay);
            }
        };
        while self.frecency.len() > self.max_size {
            self.trim_min();
        }
    }

    fn trim_min(&mut self) {
        let min_key = {
            let mut min_entry = None;
            for e in &self.frecency {
                min_entry = match min_entry {
                    None => Some(e),
                    Some(old_min) => {
                        if old_min.1 > e.1 {
                            Some(e)
                        } else {
                            Some(old_min)
                        }
                    }
                };
            }
            match min_entry {
                None => None,
                Some(e) => Some(e.0.clone()),
            }
        };

        if let Some(min) = min_key {
            self.frecency.remove(&min);
        }
    }

    pub fn items(&self) -> Vec<&T> {
        self.items_with_frecency().iter().map(|&(k, _)| k).collect()
    }

    pub fn items_with_frecency(&self) -> Vec<(&T, f64)> {
        let mut v = self.frecency.iter().map(|(ref t, f)| (*t, f.clone())).collect::<Vec<_>>();
        v.sort_unstable_by(|&(_, rhs), &(_, lhs)| {
            // Note: f64 doesn't implement ord, so we do a poor-man's ord here.
            // This is wrong for NaN, but fortunately we don't have those here.
            if lhs < rhs {
                cmp::Ordering::Less
            } else if lhs > rhs {
                cmp::Ordering::Greater
            } else {
                cmp::Ordering::Equal
            }
        });
        v
    }

    pub fn normalized_frecency(&self) -> Vec<(&T, f64)> {
        let items = self.items_with_frecency();
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
}

#[cfg(test)]
mod test {
    use super::Frecency;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::time;

    fn timef(u: u64) -> SystemTime {
        UNIX_EPOCH + time::Duration::from_secs(u)
    }

    #[test]
    fn basic_frecency_test() {
        let mut f = Frecency::<String>::new(100);
        f.visit_with_time("foo".to_string(), timef(10));
        f.visit_with_time("bar".to_string(), timef(20));
        f.visit_with_time("foo".to_string(), timef(50));
        assert_eq!(f.items(), vec![&"foo".to_string(), &"bar".to_string()]);
    }

    #[test]
    fn trims_min() {
        let mut f = Frecency::<&str>::new(2);
        f.visit_with_time("foo", timef(10));
        assert_eq!(f.items().len(), 1);
        f.visit_with_time("bar", timef(10));
        f.visit_with_time("bar", timef(10));
        f.visit_with_time("bar", timef(20));
        assert_eq!(f.items().clone(), vec![&"bar", &"foo"]);
        f.visit_with_time("baz", timef(30));
        assert_eq!(f.items().clone(), vec![&"bar", &"baz"]);
    }

    #[test]
    fn frecency_decay_works() {
        let mut f = Frecency::<&str>::new(5);
        // 1)
        // We picked a halflife of 30 days (matches mozilla)
        // That means two visits over 30 days ago should have decayed to less than one visit now
        let now = SystemTime::now();
        f.visit_with_time("foo", now - time::Duration::from_secs(31 * 24 * 60 * 60));
        f.visit_with_time("foo", now - time::Duration::from_secs(31 * 24 * 60 * 60));
        f.visit_with_time("bar", now);
        assert_eq!(f.items().clone(), vec![&"bar", &"foo"]);
    }
}
