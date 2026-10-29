
    #[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::tests::*;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::inspect_recovery_projection, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::write_field, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::CURRENT_PROJECTION_DOMAIN};

    #[test]
    fn projection_oracle_admits_current_domain_and_both_physical_variants() {
        for source_copy in [false, true] {
            assert!(inspect_recovery_projection(&projection(
                CURRENT_PROJECTION_DOMAIN,
                source_copy,
                Some(&[0])
            ))
            .is_ok());
        }
    }

    #[test]
    fn projection_oracle_denies_missing_or_malformed_current_operation() {
        for semantic in [None, Some(&[][..]), Some(&[0, 0][..]), Some(&[3][..])] {
            assert!(inspect_recovery_projection(&projection(
                CURRENT_PROJECTION_DOMAIN,
                false,
                semantic
            ))
            .is_err());
        }
        assert!(inspect_recovery_projection(&projection(
            b"store.physical.recovery-projection.v6",
            false,
            Some(&[0])
        ))
        .is_err());
        let mut bound = vec![1];
        bound.extend_from_slice(&[7; 24]);
        bound.extend_from_slice(&[8; 32]);
        bound.extend_from_slice(&5_u64.to_le_bytes());
        assert!(inspect_recovery_projection(&projection(
            CURRENT_PROJECTION_DOMAIN,
            false,
            Some(&bound)
        ))
        .is_ok());
        assert!(inspect_recovery_projection(&projection(
            CURRENT_PROJECTION_DOMAIN,
            true,
            Some(&bound)
        ))
        .is_err());
    }
