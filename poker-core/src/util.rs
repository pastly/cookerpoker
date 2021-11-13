use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn random_string(count: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(count)
        .collect()
}