//! Simulates [benford's law][https://en.wikipedia.org/wiki/Benford%27s_law]
//! 
//! `Coded on 4/14/26`

use rand::{RngExt, rngs::ThreadRng};

/// Generates a random number between 2 random magnitudes
/// and gets the the leading digit.
fn generate_leading_digit(rng: &mut ThreadRng) -> usize {
    let magnitudes = rng.random_range(1_f64..5_f64);
    let lower_magnitude = rng.random_range(0_f64..5_f64);

    let lower_bound = 10.0_f64.powf(lower_magnitude);
    let upper_bound = 10.0_f64.powf(lower_magnitude + magnitudes);

    // i promise this can never panic
    // n.to_string().chars().next() will always be Some(c)
    // where c is the first digit
    rng.random_range(lower_bound..upper_bound)
        .to_string()
        .chars()
        .next()
        .map(|s| s.to_digit(10))
        .flatten()
        .unwrap() as usize
}

/// Empirically proving
/// [benford's law][https://en.wikipedia.org/wiki/Benford%27s_law]
/// by generating a bunch of random numbers over a number of magnitudes,
/// keeping track of the frequency of the first digit, and printing out the
/// statistics.
pub fn benfords_law(trials: u32) {
    let mut rng = rand::rng();

    let mut stats: Vec<u32> = vec![0; 9];

    for _ in 0..trials {
        let leading_digit = generate_leading_digit(&mut rng);
        stats[leading_digit - 1] += 1;
    }

    for (&count, digit) in stats.iter().zip(1..) {
        let percent = (count as f64 / trials as f64) * 100.0;
        let expected = ((digit as f64 + 1.0).log10() - (digit as f64).log10())
            * 100.0;

        println!("{digit}: {percent:>5.2}% (expected {expected:>5.2}%)");
    }
}