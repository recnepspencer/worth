#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::read_contracts::*;

#[test]
fn every_delta_has_an_independent_hand_authored_read_footprint() {
    for id in DeltaId::ALL {
        assert_eq!(
            id.contract().read,
            expected_read(id),
            "read contract for {id:?}"
        );
    }
}
