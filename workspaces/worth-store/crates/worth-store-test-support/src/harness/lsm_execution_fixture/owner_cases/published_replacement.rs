//! A membership world that has crossed the real replacement lifecycle: owner
//! selection, a physically published compaction, an admitted output frame, a
//! persisted activation artifact, and an admitted in-session replacement.

use std::path::PathBuf;

use worth_store_lsm_authority::{
    admit_lsm_membership_replacement, admit_lsm_replacement_output,
    prepare_lsm_membership_activation, replace_lsm_membership, select_lsm_compaction_membership,
    LsmMembershipArtifactDeclaration, LsmMembershipKey, LsmMembershipSession,
    LsmPhysicalCompactionIntent,
};
use worth_store_wal::{
    observe_checkpoint_artifact, LogSequenceNumber, WalFrameArtifactObservation,
    WalFramePublicationScope, WalLsnRange, WalSegmentGeneration, WalSegmentId,
};

use super::super::wal_artifact_observation;
use super::world;
use crate::harness::physical_isolation::compaction::{
    admitted_compaction_plan, published_compaction_at_manifest,
};

pub(super) struct PublishedReplacementWorld {
    pub session: LsmMembershipSession,
    pub key: LsmMembershipKey,
    pub anchor: WalFrameArtifactObservation,
    /// The shared WAL segment that holds the three inputs and then the output.
    pub segment_path: PathBuf,
    pub record_frame_offsets: [u64; 3],
    pub output_frame_offset: u64,
}

pub(super) fn published_replacement() -> PublishedReplacementWorld {
    let complete = world::complete_membership();
    let mut session = complete.session;
    let selected = select_lsm_compaction_membership(&session, complete.key)
        .into_result()
        .expect("complete membership selects");
    let plan = admitted_compaction_plan();
    let manifest = plan.protected().root().manifest_epoch().get() + 1;
    let intent = LsmPhysicalCompactionIntent::from_interlock_plan(plan.clone(), manifest)
        .expect("the manifest epoch advances");
    let output_lsn = selected
        .expected_output_identity()
        .expect("output identity")
        .sequence();
    let output_scope = WalFramePublicationScope::new(
        WalSegmentId::new(1).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
        WalLsnRange::new(
            LogSequenceNumber::new(output_lsn),
            LogSequenceNumber::new(output_lsn + 1),
        )
        .unwrap(),
        selected.compaction_output_digest(
            intent.root_scope(),
            intent.target_epoch(),
            intent.manifest_epoch(),
        ),
        4096,
    )
    .expect("output scope");
    let output_frame = wal_artifact_observation(
        output_scope.clone(),
        LsmMembershipArtifactDeclaration::compaction_output(&output_scope).bytes(),
    );
    let output_frame_offset = output_frame.frame_offset();
    let output = admit_lsm_replacement_output(&selected, output_frame, intent)
        .expect("the persisted output frame admits");
    let physical = published_compaction_at_manifest(plan, manifest);
    let activation = prepare_lsm_membership_activation(&selected, output, &physical)
        .expect("the physical publication binds the output");
    let activation_bytes = activation.artifact().bytes().to_vec();
    let store_root = complete
        .anchor
        .path()
        .parent()
        .and_then(std::path::Path::parent)
        .expect("fixture WAL root");
    let activation_directory = store_root.join("worth-store-durability-lsm-activation");
    std::fs::create_dir_all(&activation_directory).unwrap();
    let activation_path = activation_directory.join("published");
    std::fs::write(&activation_path, &activation_bytes).unwrap();
    let checkpoint =
        observe_checkpoint_artifact(&activation_path, activation.scope(), &activation_bytes)
            .expect("the persisted activation is observed");
    let admitted = admit_lsm_membership_replacement(&selected, activation, checkpoint)
        .expect("the observed activation admits");
    replace_lsm_membership(&mut session, &selected, &admitted)
        .into_result()
        .expect("the admitted replacement publishes");
    PublishedReplacementWorld {
        session,
        key: complete.key,
        segment_path: complete.record_paths[0].clone(),
        record_frame_offsets: complete.record_frame_offsets,
        output_frame_offset,
        anchor: complete.anchor,
    }
}

impl PublishedReplacementWorld {
    /// Cuts the shared WAL segment at a frame boundary, so the scanner still
    /// reads a valid prefix while every later frame is gone.
    pub(super) fn truncate_segment_at(&self, frame_offset: u64) {
        std::fs::OpenOptions::new()
            .write(true)
            .open(&self.segment_path)
            .unwrap()
            .set_len(frame_offset)
            .unwrap();
    }
}
