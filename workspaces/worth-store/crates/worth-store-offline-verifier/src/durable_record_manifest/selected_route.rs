//! Independent admission of the seven selected-route bytes inside a durable
//! routing leaf. The offline oracle does not invoke the producer's decoder.

pub(super) fn selected_route_metadata_is_valid(encoded: [u8; 7], schema: u8) -> bool {
    if schema == 2 {
        return encoded == [0; 7];
    }
    if schema != 3 || encoded[5..7] != [0; 2] || encoded[4] > 2 {
        return false;
    }
    let family = u16::from_le_bytes([encoded[2], encoded[3]]);
    match encoded[0] {
        0 => encoded[1] == 0 && family == 0 && encoded[4] == 0,
        1 | 4 => encoded[1] == 0 && family == 0,
        2 => (1..=16).contains(&encoded[1]) && family == 0,
        3 => encoded[1] == 0 && family != 0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::selected_route_metadata_is_valid as valid;

    #[test]
    fn selected_route_schema_three_accepts_each_canonical_class_and_tier() {
        assert!(valid([0, 0, 0, 0, 0, 0, 0], 2));
        assert!(valid([0, 0, 0, 0, 0, 0, 0], 3));
        assert!(valid([1, 0, 0, 0, 0, 0, 0], 3));
        assert!(valid([2, 1, 0, 0, 1, 0, 0], 3));
        assert!(valid([2, 16, 0, 0, 2, 0, 0], 3));
        assert!(valid([3, 0, 7, 0, 0, 0, 0], 3));
        assert!(valid([4, 0, 0, 0, 2, 0, 0], 3));
    }

    #[test]
    fn selected_route_schema_rejects_cross_class_and_reserved_bytes() {
        for invalid in [
            [0, 0, 0, 0, 1, 0, 0],  // UnknownLegacy cannot have tier authority.
            [1, 1, 0, 0, 0, 0, 0],  // Opaque has no blob kind.
            [2, 0, 0, 0, 0, 0, 0],  // Blob kind zero is absent.
            [2, 17, 0, 0, 0, 0, 0], // Unknown blob kind.
            [2, 1, 1, 0, 0, 0, 0],  // Blob has no BTree family.
            [3, 0, 0, 0, 0, 0, 0],  // BTree family zero is absent.
            [3, 1, 1, 0, 0, 0, 0],  // BTree has no blob kind.
            [4, 0, 0, 0, 3, 0, 0],  // Unknown tier.
            [4, 0, 0, 0, 0, 1, 0],  // Reserved suffix.
        ] {
            assert!(!valid(invalid, 3), "invalid schema-3 route: {invalid:?}");
        }
        assert!(!valid([1, 0, 0, 0, 0, 0, 0], 2));
        assert!(!valid([0; 7], 4));
    }
}
