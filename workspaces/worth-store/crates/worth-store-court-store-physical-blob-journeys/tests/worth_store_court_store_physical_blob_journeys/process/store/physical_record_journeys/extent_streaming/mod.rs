


#[test]
fn extent_allocation_peak_is_independent_of_logical_record_length() {
    crate::physical_record_journeys::extent_streaming::allocation::prove();
}
#[path = "read_damage/mod.rs"]
pub(crate) mod read_damage;
#[path = "roundtrip/mod.rs"]
pub(crate) mod roundtrip;
