//! Independent checkpoint literal vectors shared by the reader courts.
#[path = "../../tests/phase_4_literal_vectors/checkpoint_records.rs"]
pub(super) mod checkpoint_records;
#[path = "../../tests/phase_4_literal_vectors/oracle.rs"]
pub(super) mod oracle;

#[test]
fn checkpoint_literals_match_their_independent_integrity_declarations() {
    checkpoint_records::verify();
}
