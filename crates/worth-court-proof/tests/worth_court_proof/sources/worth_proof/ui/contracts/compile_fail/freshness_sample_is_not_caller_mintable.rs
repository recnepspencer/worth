use std::convert::Infallible;
use worth_proof::{FreshnessSample, FreshnessSource};

struct Clock;

impl FreshnessSource for Clock {
    type Sample = u64;
    type Error = Infallible;
    fn sample(&self) -> Result<u64, Infallible> { Ok(1) }
}

fn main() {
    let _forged = FreshnessSample::<Clock> {
        value: 1,
        source: std::marker::PhantomData,
    };
}
