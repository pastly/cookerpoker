use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

pub fn random_string(count: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(count)
        .collect()
}

/// Appends content of of second hashmap to first.
/// Modifies the first in place, and consumes the second
pub fn merge_hashmap(main: &mut HashMap<i32, i32>, other: HashMap<i32, i32>) {
    for (key, value) in other {
        main.insert(key, main.get(&key).copied().unwrap_or_default() + value);
    }
}
