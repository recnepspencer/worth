use worth_proof::{CanonicalVec, NonEmpty};

fn takes_non_empty(_: NonEmpty<u8>) {}

fn takes_canonical(_: CanonicalVec<u8>) {}

fn main() {
    let raw = vec![1_u8, 2, 3];

    takes_non_empty(raw.clone());
    takes_canonical(raw);
}
