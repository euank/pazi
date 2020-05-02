use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::f64;
use std::fmt;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

const DECAY_RATE: f64 = f64::consts::LN_2 / (30. * 24. * 60. * 60.);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Frecency<T>
where
    T: Hash + Eq + Ord + Clone,
{
    // ordering is enforced on access, not on store. This is because updating an entry (visiting)
    // is a much more frequent operation than searching through items for this program.
    frecency: HashMap<T, f64>,
    max_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrecencyView<'a, T, I>
where
    T: Hash + Eq + Ord + Clone,
    T: 'a,
    I: IntoIterator<Item = (&'a T, &'a f64)>,
{
    items: I,
}

impl<T> Frecency<T>
where
    T: Hash + Eq + Ord + Clone + fmt::Debug,
{
    pub fn new(max_size: usize) -> Self {
        Frecency {
            frecency: HashMap::new(),
            max_size: max_size,
        }
    }

    pub fn visit(&mut self, key: T) {
        self.visit_with_time_and_weight(key, SystemTime::now(), 1.0)
    }

    /// Visit the given directory, and give it a 'frecency' weight bump of the provided multiplier.
    /// A weight of '2.0' indicates that this should be treated the same as the directory being
    /// visited twice at this specific time, for example.
    pub fn visit_weight(&mut self, key: T, weight: f64) {
        self.visit_with_time_and_weight(key, SystemTime::now(), weight)
    }

    // based off https://wiki.mozilla.org/User:Jesse/NewFrecency#Proposed_new_definition
    fn visit_with_time_and_weight(&mut self, key: T, now: SystemTime, weight: f64) {
        // The only error here is if the system clock is before the unix epoch. I'm fine panicing
        // there.
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let now_secs = since_epoch.as_secs() as f64 + since_epoch.subsec_nanos() as f64 * 1e-9;
        let now_decay = now_secs * DECAY_RATE;
        debug!("upserting {:?} with weight {}", key, weight);
        match self.frecency.entry(key) {
            Entry::Occupied(mut e) => {
                let frecency = e.get_mut();
                *frecency = ((*frecency - now_decay).exp() + weight).ln() + now_decay;
                debug!("Changed to {}", *frecency);
            }
            Entry::Vacant(e) => {
                debug!("Adding with ({}, {})", weight, now_decay);
                e.insert(weight.ln() + now_decay);
            }
        };
        while self.frecency.len() > self.max_size {
            self.trim_min();
        }
    }

    pub fn insert(&mut self, key: T) {
        self.insert_with_time(key, SystemTime::now())
    }

    pub fn overwrite(&mut self, key: T, value: f64) {
        self.frecency.insert(key, value);
    }

    fn insert_with_time(&mut self, key: T, now: SystemTime) {
        if !self.frecency.contains_key(&key) {
            self.visit_with_time_and_weight(key, now, 1.0)
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
            debug!("trimming: {:?}", min);
            self.frecency.remove(&min);
        }
    }

    pub fn items(&self) -> FrecencyView<T, &HashMap<T, f64>> {
        FrecencyView {
            items: &self.frecency,
        }
    }

    pub fn remove(&mut self, key: &T) -> Option<f64> {
        self.frecency.remove(key)
    }
}

impl<'a, T, I> FrecencyView<'a, T, I>
where
    T: Hash + Eq + Ord + Clone,
    T: 'a,
    I: IntoIterator<Item = (&'a T, &'a f64)>,
{
    pub fn normalized(self) -> Vec<(&'a T, f64)> {
        let mut items: Vec<_> = self
            .items
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .collect();
        if items.len() == 0 {
            return Vec::new();
        }
        items.sort_by(descending_frecency);
        let min = items[items.len() - 1].1;
        let max = items[0].1;
        items
            .into_iter()
            .map(|(s, v)| {
                let normalized = (v - min) / (max - min);
                if normalized.is_nan() {
                    (s, 0.0)
                } else {
                    (s, normalized)
                }
            })
            .collect()
    }

    pub fn raw(self) -> Vec<(&'a T, f64)> {
        self.items
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .collect()
    }
}

pub fn descending_frecency<T>(lhs: &(T, f64), rhs: &(T, f64)) -> Ordering {
    // NaN shouldn't happen
    rhs.1
        .partial_cmp(&lhs.1)
        .expect(&format!("{} could not be compared to {}", lhs.1, rhs.1))
}

#[cfg(test)]
mod test {
    use super::{Frecency, FrecencyView};
    use std;
    use std::hash::Hash;
    use std::time;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn timef(u: u64) -> SystemTime {
        UNIX_EPOCH + time::Duration::from_secs(u)
    }

    fn keys<'a, T, I>(f: FrecencyView<'a, T, I>) -> Vec<T>
    where
        I: IntoIterator<Item = (&'a T, &'a f64)>,
        T: 'a,
        T: Ord + Clone + Hash + std::fmt::Debug,
    {
        f.normalized().into_iter().map(|(k, _)| k.clone()).collect()
    }

    #[test]
    fn basic_frecency_test() {
        let mut f = Frecency::<&str>::new(100);
        f.visit_with_time_and_weight("foo", timef(10), 1.0);
        f.visit_with_time_and_weight("bar", timef(20), 1.0);
        f.visit_with_time_and_weight("foo", timef(50), 1.0);
        f.insert_with_time("quux", timef(70));
        assert_eq!(keys(f.items()), vec!["foo", "quux", "bar"]);
        let f_clone = f.clone();
        f.insert_with_time("quux", timef(77));
        assert_eq!(f_clone.items(), f.items());
    }

    #[test]
    fn trims_min() {
        let mut f = Frecency::<&str>::new(2);
        f.visit_with_time_and_weight("foo", timef(10), 1.0);
        assert_eq!(f.items().normalized().len(), 1);
        f.visit_with_time_and_weight("bar", timef(10), 1.0);
        f.visit_with_time_and_weight("bar", timef(10), 1.0);
        f.visit_with_time_and_weight("bar", timef(20), 1.0);
        assert_eq!(keys(f.items()), vec!["bar", "foo"]);
        f.visit_with_time_and_weight("baz", timef(30), 1.0);
        assert_eq!(keys(f.items()), vec!["bar", "baz"]);
    }

    #[test]
    fn frecency_decay_works() {
        let mut f = Frecency::<&str>::new(5);
        // 1)
        // We picked a halflife of 30 days (matches mozilla)
        // That means two visits over 30 days ago should have decayed to less than one visit now
        let now = SystemTime::now();
        f.visit_with_time_and_weight("foo", now - time::Duration::from_secs(31 * 24 * 60 * 60), 1.0);
        f.visit_with_time_and_weight("foo", now - time::Duration::from_secs(31 * 24 * 60 * 60), 1.0);
        f.visit_with_time_and_weight("bar", now, 1.0);
        assert_eq!(keys(f.items()), vec!["bar", "foo"]);
    }

    #[test]
    fn frecency_weight_works() {
        let mut f = Frecency::<&str>::new(5);
        // Visit with a weight of 5x should mean that older is 'better' than newer until we visit
        // newer 5 more times
        f.visit_with_time_and_weight("older", timef(10), 5.0);
        f.visit_with_time_and_weight("newer", timef(11), 1.0);
        assert_eq!(keys(f.items()), vec!["older", "newer"]);
        for _ in 0..3 {
            f.visit_with_time_and_weight("newer", timef(11), 1.0);
        }
        // still shouldn't beat 5 weight
        assert_eq!(keys(f.items()), vec!["older", "newer"]);
        for _ in 0..2 {
            f.visit_with_time_and_weight("newer", timef(11), 1.0);
        }
        // 2 more times makes 1 + 3 + 2 = 6, should now trump the 5x weighting
        assert_eq!(keys(f.items()), vec!["newer", "older"]);
    }
}
