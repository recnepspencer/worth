//! Bounded M2-style prefixes: only complete operations precede a restart.
#[derive(Clone, Copy, Debug)]
pub(super) enum Restart {
    Close,
    QuiescentKill,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Outcome {
    Committed,
    IntentDrift,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Step {
    pub key: u64,
    pub value: u64,
    pub outcome: Outcome,
    pub restart: Restart,
    pub handler_calls: usize,
}
pub(super) fn generate(seed: u64) -> Vec<Step> {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    };
    let first_key = next();
    let mut steps = Vec::new();
    for group in 0..4u64 {
        let key = first_key.wrapping_add(group);
        let value = next() % 100_000;
        for retry in 0..3 {
            steps.push(Step {
                key,
                value: value + u64::from(retry == 2),
                outcome: if retry == 2 {
                    Outcome::IntentDrift
                } else {
                    Outcome::Committed
                },
                restart: if (steps.len() as u64).wrapping_add(seed) & 1 == 0 {
                    Restart::Close
                } else {
                    Restart::QuiescentKill
                },
                handler_calls: usize::from(retry == 0),
            });
        }
    }
    steps
}
