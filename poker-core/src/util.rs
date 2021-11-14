use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn random_string(count: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(count)
        .collect()
}
