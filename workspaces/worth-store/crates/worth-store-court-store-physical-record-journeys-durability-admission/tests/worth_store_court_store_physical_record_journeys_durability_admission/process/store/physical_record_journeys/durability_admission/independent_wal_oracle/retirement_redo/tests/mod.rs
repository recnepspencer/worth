
    #[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::tests::*;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::BODY_BYTES, worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementRedo, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::REWRITE_DOMAIN, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::inspect_retirement_redo};

    #[test]
    fn retirement_golden_vectors_reject_unknown_and_malformed_payloads() {
        assert_eq!(golden_intent().len(), 8 + 28 + BODY_BYTES);
        assert_eq!(
            inspect_retirement_redo(&golden_intent()),
            Ok(IndependentRetirementRedo {
                action: worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
                kind: worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Segment,
                source_root: 4,
                artifact_id: 1,
                generation: 2,
                bytes: 16,
                arena_range: None,
                release_roots: None,
                release_digest: None,
                metadata_bytes: 0,
                publication: 0,
            })
        );
        let mut completion = golden_intent();
        completion[36] = 2;
        assert_eq!(
            inspect_retirement_redo(&completion).unwrap().action,
            worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion
        );

        let mut truncated = golden_intent();
        truncated.pop();
        assert_eq!(
            inspect_retirement_redo(&truncated),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated)
        );
        let mut trailing = golden_intent();
        trailing.push(0);
        assert_eq!(
            inspect_retirement_redo(&trailing),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::TrailingBytes)
        );
        let mut extent_intent = golden_intent();
        extent_intent[36] = 3;
        extent_intent[69..77].copy_from_slice(&3u64.to_le_bytes());
        extent_intent[77..85].copy_from_slice(&4096u64.to_le_bytes());
        extent_intent[85..93].copy_from_slice(&16u64.to_le_bytes());
        extent_intent[93..101].copy_from_slice(&8u64.to_le_bytes());
        extent_intent[101..109].copy_from_slice(&9u64.to_le_bytes());
        extent_intent[109..141].fill(0x42);
        extent_intent[141..149].copy_from_slice(&4096u64.to_le_bytes());
        extent_intent[149..157].copy_from_slice(&71u64.to_le_bytes());
        let decoded = inspect_retirement_redo(&extent_intent).unwrap();
        assert_eq!(
            (decoded.action, decoded.kind),
            (
                worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
                worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent
            )
        );
        let mut extent_completion = extent_intent.clone();
        extent_completion[36] = 4;
        let decoded = inspect_retirement_redo(&extent_completion).unwrap();
        assert_eq!(
            (decoded.action, decoded.kind),
            (
                worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
                worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent
            )
        );
        let mut unknown_action = golden_intent();
        unknown_action[36] = 7;
        assert_eq!(
            inspect_retirement_redo(&unknown_action),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame)
        );
        let mut rewrite = (REWRITE_DOMAIN.len() as u64).to_le_bytes().to_vec();
        rewrite.extend_from_slice(REWRITE_DOMAIN);
        assert_eq!(
            inspect_retirement_redo(&rewrite),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch)
        );
    }
