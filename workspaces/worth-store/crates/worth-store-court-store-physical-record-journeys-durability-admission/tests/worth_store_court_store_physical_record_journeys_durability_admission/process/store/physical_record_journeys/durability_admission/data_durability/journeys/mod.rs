#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::data_durability::journeys::*;
use {std::collections::BTreeSet};
use {std::fs};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {crate::durability_admission::data_durability::mutation_world::append, crate::durability_admission::data_durability::mutation_world::synchronize};
use {crate::physical_record_journeys::durability_admission::data_durability::oracle::assert_encoded_wal_matches_claims, crate::physical_record_journeys::durability_admission::data_durability::oracle::assert_targets_absent, crate::physical_record_journeys::durability_admission::data_durability::oracle::frame_page_lsn_matches_basis, crate::physical_record_journeys::durability_admission::data_durability::oracle::verify_effect};
use {worth_store::physical_runtime::CertifiedPriorPageImage, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataEffectSource, worth_store::physical_runtime::PhysicalDataFrameSubject, worth_store::physical_runtime::PhysicalDataSettlementOutcome, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::PhysicalPageLsn};

#[test]
fn materialized_inline_prior_advances_through_exact_wal_bound_copy_on_write() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);
    let (format, placement, _) = configuration();
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("data-durability-materialized-prior", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"published-prior".as_slice()]).unwrap(),
    );
    let submission = serving.certification_record_submission();
    let source_path = only_data_artifact(&store_root, "segments");
    let source_before = std::fs::read(&source_path).unwrap();

    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([101; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"copy-on-write-delta".as_slice()]).unwrap(),
    );
    let appended_member = appended.members()[0].mutation();
    assert_encoded_wal_matches_claims(&store_root, appended_member);
    assert_targets_absent(&store_root, appended_member);
    let member = appended_member.reserved().member_basis();
    let durable = synchronize(&submission, appended);
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("the exact WAL-durable copy-on-write image must dispatch"),
    };
    assert_eq!(dispatched.effects().len(), 1);
    let effect = &dispatched.effects()[0];
    assert_eq!(effect.source(), worth_store::physical_runtime::PhysicalDataEffectSource::NewArtifact);
    let prior = effect.basis().prior();
    assert!(matches!(
        prior.image(),
        worth_store::physical_runtime::CertifiedPriorPageImage::MaterializedSource(_)
    ));
    assert_eq!(effect.basis().delta().len(), 1);
    let target_bytes = verify_effect(&store_root, format.declaration(), member, effect);
    let mut page_lsn_ahead = target_bytes.clone();
    encode_data_frame_page_lsn(
        &mut page_lsn_ahead,
        worth_store_physical_format::DurableFrameKind::InlinePage,
        worth_store_physical_format::PhysicalPageLsn::new(effect.basis().resulting_page_lsn().get() + 1),
    )
    .expect("controlled pageLSN-ahead frame remains structurally valid");
    assert!(
        !frame_page_lsn_matches_basis(effect, &page_lsn_ahead),
        "the independent oracle must reject a checksum-valid pageLSN-ahead mutation"
    );
    assert_ne!(target_bytes, source_before);
    assert_eq!(std::fs::read(&source_path).unwrap(), source_before);
    match dispatched.settle_exact_effects() {
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(settled) => {
            assert_eq!(settled.mutation_identity(), member.mutation_identity())
        }
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::InspectionRequired { cause, .. } => {
            panic!("exact copy-on-write effects must settle: {cause:?}")
        }
    }
    serving.close();
}

#[test]
fn multi_chunk_extent_writes_distinct_new_frames_within_one_packed_arena() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);
    let (format, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let payload = vec![0x5a; format.declaration().page_size().bytes() as usize * 3];
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([102; 32]),
        worth_store::physical_runtime::RecordAppendBatch::builder()
            .push_owned(payload)
            .build()
            .unwrap(),
    );
    let appended_member = appended.members()[0].mutation();
    assert_encoded_wal_matches_claims(&store_root, appended_member);
    assert_targets_absent(&store_root, appended_member);
    let member = appended_member.reserved().member_basis();
    let durable = synchronize(&submission, appended);
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("the exact WAL-durable extent must dispatch"),
    };
    assert!(
        dispatched.effects().len() >= 3,
        "the fixture must cross the real multi-chunk packed-arena join"
    );
    let mut artifacts = std::collections::BTreeSet::new();
    let mut ranges = Vec::new();
    for effect in dispatched.effects() {
        assert_eq!(effect.source(), worth_store::physical_runtime::PhysicalDataEffectSource::NewArtifact);
        assert!(matches!(
            effect.basis().target().subject(),
            worth_store::physical_runtime::PhysicalDataFrameSubject::ExtentChunk(_)
        ));
        assert!(matches!(
            effect.basis().prior().image(),
            worth_store::physical_runtime::CertifiedPriorPageImage::AbsentTarget(target)
                if target == effect.basis().target()
        ));
        assert_eq!(effect.basis().delta().len(), 1);
        artifacts.insert(effect.coordinate().artifact());
        ranges.push((
            effect.coordinate().offset(),
            effect.coordinate().offset() + u64::from(effect.coordinate().length()),
        ));
        verify_effect(&store_root, format.declaration(), member, effect);
    }
    assert_eq!(artifacts.len(), 1);
    assert!(ranges.windows(2).all(|pair| pair[0].1 <= pair[1].0));
    match dispatched.settle_exact_effects() {
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(settled) => {
            assert_eq!(settled.mutation_identity(), member.mutation_identity())
        }
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::InspectionRequired { cause, .. } => {
            panic!("exact packed-arena frame effects must settle: {cause:?}")
        }
    }
    serving.close();
}
