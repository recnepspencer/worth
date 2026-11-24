//! Law-based property suites for Store layout indexes:
//! - Fixture-managed multi-leaf `BTreeNodeV1` codec, child/sibling link routing,
//!   and bounded backup verifier oracle vs `BTreeMap`
//! - `BlobCatalogPointKey` total-order isomorphism, byte boundaries & zero-denial laws
//! - Bounded layout index backup verification security & digest laws
//! - Access-shape & degraded-scan budget admission + over-budget denial laws
#[path = "btree_model.rs"]
mod btree_model;

use self::btree_model::{hex_digest, IndexOp, PersistedBTreeModel};
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::io::Cursor;
use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_layout_indexes::{
    access_shapes, layout_degraded_scan_runtime, verify_bounded_layout_index_artifact_from_reader,
    AccessLaneClassification, AccessPlanSelectionDenied, BlobCatalogPointKey,
    BlobCatalogPointKeyDenial, BoundedLayoutIndexDenial, BoundedLayoutIndexVerificationRequest,
    DegradedExactScanExecutionDenied, DegradedExactScanExecutionRequest, FullDeclaredScanBasis,
    FullDeclaredScanView, LayoutIndexBackupFormat, PlannedCounterObservation,
};
use worth_store_physical_format::{
    extent_payload_digest, BTreeNodeCellV1, BTreeNodeDenial, BTreeNodeV1, PhysicalPageId,
    PhysicalSegmentId,
};
use worth_store_security::admitted_tenant_page_security_scope_for_layout_partition_test;

use crate::degraded_scan_ordinary_runtime::physical_fixture::{admitted_catalog, open_runtime};

fn index_op_strategy() -> impl Strategy<Value = IndexOp> {
    prop_oneof![
        4 => (0_u8..40, prop::collection::vec(any::<u8>(), 1..=24))
            .prop_map(|(key, value)| IndexOp::InsertOrUpdate { key, value }),
        2 => (0_u8..40).prop_map(|key| IndexOp::Delete { key }),
        2 => (0_u8..40, 0_u8..40).prop_map(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            IndexOp::RangeScan { lo, hi }
        }),
    ]
}

fn boundary_generation_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        2 => prop::sample::select(vec![
            0_u64,
            1,
            0xff,
            0x100,
            0xffff,
            0x1_0000,
            0xff_ffff,
            0x100_0000,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
            u64::MAX - 1,
            u64::MAX,
        ]),
        1 => any::<u64>(),
    ]
}

fn nonzero_object_strategy() -> impl Strategy<Value = [u8; 16]> {
    prop::array::uniform16(any::<u8>()).prop_filter("non-zero object id", |o| *o != [0; 16])
}

fn blob_key_pair_strategy() -> impl Strategy<Value = ([u8; 16], u64, [u8; 16], u64)> {
    prop_oneof![
        // Same non-zero object with distinct / boundary generations.
        3 => (
            nonzero_object_strategy(),
            boundary_generation_strategy(),
            boundary_generation_strategy(),
        )
            .prop_map(|(obj, g1, g2)| (obj, g1, obj, g2)),
        // Close objects differing at one byte position.
        2 => (
            nonzero_object_strategy(),
            0_usize..16,
            any::<u8>(),
            boundary_generation_strategy(),
            boundary_generation_strategy(),
        )
            .prop_map(|(obj_a, idx, delta, g1, g2)| {
                let mut obj_b = obj_a;
                obj_b[idx] = obj_b[idx].wrapping_add(delta);
                (obj_a, g1, obj_b, g2)
            }),
        // Explicit zero-object or zero-generation inputs.
        1 => (
            prop_oneof![Just([0_u8; 16]), nonzero_object_strategy()],
            prop_oneof![Just(0_u64), boundary_generation_strategy()],
            prop_oneof![Just([0_u8; 16]), nonzero_object_strategy()],
            prop_oneof![Just(0_u64), boundary_generation_strategy()],
        ),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn btree_node_codec_routing_links_and_range_scans_match_btreemap_oracle(
        max_leaf_cells in 2_usize..=5,
        ops in prop::collection::vec(index_op_strategy(), 1..=36),
    ) {
        let mut model = PersistedBTreeModel::new(max_leaf_cells);
        let mut reference: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

        for op in &ops {
            model.apply(op);
            match op {
                IndexOp::InsertOrUpdate { key, value } => {
                    reference.insert(vec![*key], value.clone());
                    prop_assert_eq!(model.get(&[*key]), Some(value.clone()));
                }
                IndexOp::Delete { key } => {
                    reference.remove(&vec![*key]);
                    prop_assert_eq!(model.get(&[*key]), None);
                }
                IndexOp::RangeScan { lo, hi } => {
                    let expected: Vec<_> = reference
                        .range(vec![*lo]..=vec![*hi])
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    prop_assert_eq!(model.range_scan(&[*lo], &[*hi]), expected);
                }
            }
        }

        let full_expected: Vec<_> = reference
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        prop_assert_eq!(model.range_scan(&[0], &[u8::MAX]), full_expected);
    }

    #[test]
    fn btree_node_rejects_unsorted_duplicate_keys_and_forged_backup_artifacts(
        keys in prop::collection::btree_set(prop::collection::vec(any::<u8>(), 1..=16), 2..=8),
        values in prop::collection::vec(prop::collection::vec(any::<u8>(), 1..=16), 8),
        flip_pos in any::<usize>(),
        flip_bit in 0_u8..8,
    ) {
        let sorted_keys: Vec<_> = keys.into_iter().collect();
        let cells: Vec<_> = sorted_keys
            .iter()
            .zip(values.iter())
            .map(|(k, v)| BTreeNodeCellV1::leaf(k.clone(), v.clone()))
            .collect();

        let mut reversed = cells.clone();
        reversed.reverse();
        prop_assert_eq!(
            BTreeNodeV1::leaf(1, reversed, None, None),
            Err(BTreeNodeDenial::UnsortedKeys)
        );

        let dup = vec![cells[0].clone(), cells[0].clone()];
        prop_assert_eq!(
            BTreeNodeV1::leaf(1, dup, None, None),
            Err(BTreeNodeDenial::UnsortedKeys)
        );

        let encoded = BTreeNodeV1::leaf(1, cells, None, None)
            .unwrap()
            .encode(4096)
            .unwrap();
        let digest = extent_payload_digest(&encoded);
        let identity = format!("index:sha256:{}", hex_digest(digest));

        let small_req = BoundedLayoutIndexVerificationRequest::new(
            LayoutIndexBackupFormat::BTreeNodeV1,
            &identity,
            encoded.len() as u64,
            digest,
            encoded.len() - 1,
        );
        prop_assert!(
            matches!(
                verify_bounded_layout_index_artifact_from_reader(
                    &mut Cursor::new(&encoded),
                    encoded.len() as u64,
                    small_req,
                ),
                Err(BoundedLayoutIndexDenial::BufferTooSmall { .. })
            ),
            "buffer smaller than node must fail with BufferTooSmall"
        );

        let mut corrupted = encoded.clone();
        let idx = flip_pos % corrupted.len();
        corrupted[idx] ^= 1_u8 << flip_bit;
        let orig_req = BoundedLayoutIndexVerificationRequest::new(
            LayoutIndexBackupFormat::BTreeNodeV1,
            &identity,
            corrupted.len() as u64,
            digest,
            corrupted.len(),
        );
        prop_assert!(matches!(
            verify_bounded_layout_index_artifact_from_reader(
                &mut Cursor::new(&corrupted),
                corrupted.len() as u64,
                orig_req,
            ),
            Err(BoundedLayoutIndexDenial::DigestMismatch)
        ));

        let forged_digest = extent_payload_digest(&corrupted);
        let forged_id = format!("index:sha256:{}", hex_digest(forged_digest));
        let forged_req = BoundedLayoutIndexVerificationRequest::new(
            LayoutIndexBackupFormat::BTreeNodeV1,
            &forged_id,
            corrupted.len() as u64,
            forged_digest,
            corrupted.len(),
        );
        prop_assert!(matches!(
            verify_bounded_layout_index_artifact_from_reader(
                &mut Cursor::new(&corrupted),
                corrupted.len() as u64,
                forged_req,
            ),
            Err(BoundedLayoutIndexDenial::MalformedIndex)
        ));
    }

    #[test]
    fn blob_catalog_point_key_preserves_lexicographic_order_and_zero_denials(
        (obj_a, gen_a, obj_b, gen_b) in blob_key_pair_strategy(),
    ) {
        // Explicit zero-object and zero-generation denial laws on every generated sample.
        prop_assert_eq!(
            BlobCatalogPointKey::admit([0; 16], gen_a.max(1)),
            Err(BlobCatalogPointKeyDenial::EmptyObject)
        );
        if obj_a != [0; 16] {
            prop_assert_eq!(
                BlobCatalogPointKey::admit(obj_a, 0),
                Err(BlobCatalogPointKeyDenial::ZeroGeneration)
            );
        }

        let res_a = BlobCatalogPointKey::admit(obj_a, gen_a);
        if obj_a == [0; 16] {
            prop_assert_eq!(res_a, Err(BlobCatalogPointKeyDenial::EmptyObject));
            return Ok(());
        }
        if gen_a == 0 {
            prop_assert_eq!(res_a, Err(BlobCatalogPointKeyDenial::ZeroGeneration));
            return Ok(());
        }
        let ka = res_a.unwrap();

        let res_b = BlobCatalogPointKey::admit(obj_b, gen_b);
        if obj_b == [0; 16] {
            prop_assert_eq!(res_b, Err(BlobCatalogPointKeyDenial::EmptyObject));
            return Ok(());
        }
        if gen_b == 0 {
            prop_assert_eq!(res_b, Err(BlobCatalogPointKeyDenial::ZeroGeneration));
            return Ok(());
        }
        let kb = res_b.unwrap();

        let expected_ord = (obj_a, gen_a).cmp(&(obj_b, gen_b));
        prop_assert_eq!(ka.cmp(&kb), expected_ord);
        prop_assert_eq!(ka.bytes().cmp(&kb.bytes()), expected_ord);
    }

    #[test]
    fn degraded_scan_access_shapes_and_budget_admission_laws(
        seg_raw in 1_u64..=100,
        page_raw in 1_u64..=100,
        within_budget in 2_u64..=16,
        excess_budget in 5_000_u64..=50_000,
    ) {
        prop_assert!(matches!(
            access_shapes()
                .full_declared_scan(
                    AccessLaneClassification::Foreground,
                    FullDeclaredScanBasis::DeclaredFullTraversal,
                )
                .view(),
            FullDeclaredScanView::HiddenDenied(_)
        ));
        prop_assert!(matches!(
            access_shapes()
                .full_declared_scan(
                    AccessLaneClassification::Verifier,
                    FullDeclaredScanBasis::DeclaredFullTraversal,
                )
                .view(),
            FullDeclaredScanView::Success(_)
        ));

        let catalog = admitted_catalog();
        let security = admitted_tenant_page_security_scope_for_layout_partition_test();
        let mut physical = open_runtime();
        physical.publish_physical_root().unwrap();

        // Exact-budget (budget = 1) request must admit with PlannedCounterObservation::Exact.
        let exact_exec = layout_degraded_scan_runtime()
            .execute(
                DegradedExactScanExecutionRequest::new(
                    &catalog,
                    security.witnesses(),
                    PhysicalSegmentId::from_raw(seg_raw).unwrap(),
                    PhysicalPageId::from_raw(page_raw).unwrap(),
                    1,
                    PreExecutionBudgetEnvelope::terminal_default(),
                ),
                &mut physical,
            )
            .expect("exact-budget degraded scan must admit and execute");
        prop_assert_eq!(exact_exec.observed_rows(), 1);
        prop_assert_eq!(exact_exec.physical_observation().scan().counters().scans(), 1);
        prop_assert!(exact_exec.physical_observation().scan().is_budget_exact());
        prop_assert_eq!(
            exact_exec.counter_receipt().observation(),
            PlannedCounterObservation::Exact
        );

        // Within-envelope (budget > 1) request must admit with PlannedCounterObservation::WithinEnvelope.
        let envelope_exec = layout_degraded_scan_runtime()
            .execute(
                DegradedExactScanExecutionRequest::new(
                    &catalog,
                    security.witnesses(),
                    PhysicalSegmentId::from_raw(seg_raw).unwrap(),
                    PhysicalPageId::from_raw(page_raw).unwrap(),
                    within_budget,
                    PreExecutionBudgetEnvelope::terminal_default(),
                ),
                &mut physical,
            )
            .expect("within-envelope degraded scan must admit and execute");
        prop_assert_eq!(envelope_exec.observed_rows(), 1);
        prop_assert_eq!(
            envelope_exec.physical_observation().scan().counters().scans(),
            2
        );
        prop_assert!(envelope_exec.physical_observation().scan().is_budget_exact());
        prop_assert_eq!(
            envelope_exec.counter_receipt().observation(),
            PlannedCounterObservation::WithinEnvelope
        );

        // Over-budget request must fail before physical execution and leave counters unchanged.
        let before = physical.counters();
        let denial = layout_degraded_scan_runtime()
            .execute(
                DegradedExactScanExecutionRequest::new(
                    &catalog,
                    security.witnesses(),
                    PhysicalSegmentId::from_raw(seg_raw).unwrap(),
                    PhysicalPageId::from_raw(page_raw).unwrap(),
                    excess_budget,
                    PreExecutionBudgetEnvelope::foreground_default(),
                ),
                &mut physical,
            )
            .unwrap_err();
        prop_assert!(matches!(
            denial,
            DegradedExactScanExecutionDenied::Selection(
                AccessPlanSelectionDenied::BudgetDenied(_)
            )
        ));
        prop_assert_eq!(physical.counters(), before);
    }
}
