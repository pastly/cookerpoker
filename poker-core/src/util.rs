use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

pub fn _random_string(count: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(count)
        .collect()
}

/// Appends content of of second hashmap to first.
/// Modifies the first in place, and consumes the second
pub fn merge_hashmap<K, V>(main: &mut HashMap<K, V>, other: HashMap<K, V>)
where
    K: Eq + Copy + std::hash::Hash,
    V: Copy + Default + std::ops::Add<Output = V>,
{
    for (key, value) in other {
        main.insert(key, main.get(&key).copied().unwrap_or_default() + value);
    }
}
