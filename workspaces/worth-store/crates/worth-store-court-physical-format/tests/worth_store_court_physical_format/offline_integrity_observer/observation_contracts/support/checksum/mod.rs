

#[allow(unused_imports)] use crate::support::checksum::*;
#[test]
fn independent_fixture_crc_matches_the_published_check_value() {
    assert_eq!(checksum(&[b"123456789"]), 0xe306_9283);
}
