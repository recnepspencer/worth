//! Independent checkpoint literal verification shared with the reader courts.
use crate::literal_vectors::checkpoint_records;

#[test]
fn checkpoint_literals_match_their_independent_integrity_declarations() {
    checkpoint_records::verify();
}
