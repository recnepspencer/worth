use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    CertifiedPriorPageImage, PhysicalDataEffectSettlement, PhysicalDataFrameIdentity,
    PhysicalDataFrameSubject, PhysicalWalMemberBasis, WalAppendedPhysicalMutation,
};
use worth_store_physical_format::{
    decode_data_frame_page_lsn, decode_extent_chunk, inspect_inline_page, DurableFrameKind,
    PhysicalPageLsn, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::super::independent_wal_oracle::{
    independent_canonical_redo, independent_frame_payload, independent_recovery_projection,
    independent_target_claim, split_member_payload,
};

pub(super) fn assert_encoded_wal_matches_claims(
    store_root: &Path,
    appended: &WalAppendedPhysicalMutation,
) {
    let declaration = appended.reserved().declaration();
    let artifact = store_root.join("families").join("wal").join(format!(
        "segment-{}-generation-{}.wal",
        declaration.segment().get(),
        declaration.generation().get()
    ));
    let bytes = fs::read(artifact).expect("the real WAL artifact must exist");
    let range = declaration.artifact_range();
    let start = usize::try_from(range.offset()).expect("test WAL offset fits memory");
    let end = start + usize::try_from(range.byte_count()).expect("test WAL frame fits memory");
    let payload =
        independent_frame_payload(&bytes[start..end]).expect("independent WAL frame decoding");
    let (_, encoded_redo) =
        split_member_payload(payload).expect("independent member envelope decoding");
    let redo = appended.reserved().redo();
    let records = redo
        .records()
        .iter()
        .map(|record| record.bytes())
        .collect::<Vec<_>>();
    let targets = redo
        .records()
        .iter()
        .map(|record| {
            record
                .targets()
                .iter()
                .copied()
                .map(independent_target_claim)
                .collect()
        })
        .collect::<Vec<_>>();
    let projection = independent_recovery_projection(encoded_redo)
        .expect("the independent WAL oracle must admit the mandatory recovery projection");
    let expected = independent_canonical_redo(
        &records,
        declaration.lsn_range().start().get(),
        &targets,
        projection,
    );
    assert_eq!(
        encoded_redo, expected,
        "raw WAL must encode the exact logical data-image targets"
    );
}

pub(super) fn assert_targets_absent(store_root: &Path, appended: &WalAppendedPhysicalMutation) {
    for record in appended.reserved().redo().records() {
        for claim in record.targets() {
            assert!(
                !artifact_path(store_root, claim.target().coordinate().artifact()).exists(),
                "WAL append must not create a data artifact before the barrier and dispatch"
            );
        }
    }
}

pub(super) fn verify_effect(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    member: PhysicalWalMemberBasis,
    effect: &PhysicalDataEffectSettlement,
) -> Vec<u8> {
    let basis = effect.basis();
    assert_eq!(effect.coordinate(), basis.target().coordinate());
    assert_eq!(effect.payload_digest(), basis.resulting_payload_digest());
    let delta = basis.delta();
    assert!(!delta.is_empty(), "every data image requires new redo");
    assert_eq!(
        delta.last().unwrap().lsn().get(),
        basis.resulting_page_lsn().get()
    );
    assert!(delta
        .iter()
        .all(|redo| member.lsn_range().contains(redo.lsn())));
    let bytes = read_frame(store_root, basis.target());
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&bytes)),
        basis.resulting_payload_digest()
    );
    assert_eq!(
        decode_data_frame_page_lsn(&bytes, durable_kind(basis.target())),
        Ok(basis.resulting_page_lsn())
    );
    assert_identity_matches_bytes(format, basis.target(), &bytes);
    verify_prior(store_root, format, basis.prior(), basis.target());
    bytes
}

pub(super) fn frame_page_lsn_matches_basis(
    effect: &PhysicalDataEffectSettlement,
    bytes: &[u8],
) -> bool {
    decode_data_frame_page_lsn(bytes, durable_kind(effect.basis().target()))
        == Ok(effect.basis().resulting_page_lsn())
}

pub(super) fn artifact_path(store_root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    let family = match artifact {
        RecordArtifactFile::Segment { .. } => "segments",
        RecordArtifactFile::ExtentArena { .. } => "arenas",
        _ => panic!("Phase 4 data effects target only page or extent artifacts"),
    };
    store_root
        .join("families")
        .join("records")
        .join(family)
        .join(artifact.file_name())
}

fn verify_prior(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    prior: worth_store::physical_runtime::CertifiedPriorPageBasis,
    target: PhysicalDataFrameIdentity,
) {
    match prior.image() {
        CertifiedPriorPageImage::AbsentTarget(absent) => {
            assert_eq!(absent, target);
            assert_eq!(prior.page_lsn(), PhysicalPageLsn::GENESIS);
        }
        CertifiedPriorPageImage::MaterializedSource(source) => {
            let bytes = read_frame(store_root, source);
            assert_identity_matches_bytes(format, source, &bytes);
            assert_eq!(
                <[u8; 32]>::from(Sha256::digest(&bytes)),
                prior.payload_digest()
            );
            assert_eq!(
                decode_data_frame_page_lsn(&bytes, durable_kind(source)),
                Ok(prior.page_lsn())
            );
            assert_exact_inline_successor(source, target);
        }
    }
}

fn assert_exact_inline_successor(
    source: PhysicalDataFrameIdentity,
    target: PhysicalDataFrameIdentity,
) {
    let (
        PhysicalDataFrameSubject::InlinePage(source_page),
        PhysicalDataFrameSubject::InlinePage(target_page),
    ) = (source.subject(), target.subject())
    else {
        panic!("only inline pages currently reuse a materialized prior image");
    };
    assert_eq!(source_page.segment_id(), target_page.segment_id());
    assert_eq!(source_page.page_id(), target_page.page_id());
    assert_eq!(
        source_page.generation().get().checked_add(1),
        Some(target_page.generation().get())
    );
    let (
        RecordArtifactFile::Segment {
            segment: source_segment,
            generation: source_generation,
        },
        RecordArtifactFile::Segment {
            segment: target_segment,
            generation: target_generation,
        },
    ) = (
        source.coordinate().artifact(),
        target.coordinate().artifact(),
    )
    else {
        panic!("inline page images require segment artifacts");
    };
    assert_eq!(source_segment, target_segment);
    assert_eq!(source_generation.checked_add(1), Some(target_generation));
}

fn assert_identity_matches_bytes(
    format: PhysicalRecordFormatDeclaration,
    identity: PhysicalDataFrameIdentity,
    bytes: &[u8],
) {
    match identity.subject() {
        PhysicalDataFrameSubject::InlinePage(page) => assert_eq!(
            inspect_inline_page(format, bytes)
                .expect("persisted inline page must decode")
                .page_cell(),
            page
        ),
        PhysicalDataFrameSubject::ExtentChunk(chunk) => {
            let (_, found_format) =
                decode_extent_chunk(bytes, chunk).expect("persisted extent chunk must decode");
            assert_eq!(found_format, format);
        }
    }
}

fn read_frame(store_root: &Path, identity: PhysicalDataFrameIdentity) -> Vec<u8> {
    let coordinate = identity.coordinate();
    let artifact =
        fs::read(artifact_path(store_root, coordinate.artifact())).expect("data artifact exists");
    let start = usize::try_from(coordinate.offset()).expect("test frame offset fits memory");
    let end = start + usize::try_from(coordinate.length()).expect("test frame length fits memory");
    artifact
        .get(start..end)
        .expect("effect coordinate is bounded by the artifact")
        .to_vec()
}

const fn durable_kind(identity: PhysicalDataFrameIdentity) -> DurableFrameKind {
    match identity.subject() {
        PhysicalDataFrameSubject::InlinePage(_) => DurableFrameKind::InlinePage,
        PhysicalDataFrameSubject::ExtentChunk(_) => DurableFrameKind::Extent,
    }
}
