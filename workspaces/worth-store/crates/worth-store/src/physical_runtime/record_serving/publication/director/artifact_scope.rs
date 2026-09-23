use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::PhysicalOperationAllocationScope;
use worth_store_physical_format::{CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement};

use super::durable_preparation::{map_record_denial, PhysicalMutationPreparationAdmission};
use super::selected_segment_rewrite::admitted_terminal;
use super::RecordPublicationDirector;
use crate::physical_runtime::durability::PhysicalMutationOperationFamily;
use crate::physical_runtime::record_serving::access::manifest_routing::{
    ManifestRangeCursor, ManifestReader,
};
use crate::physical_runtime::record_serving::planning::inline_plan_failure::manifest_lookup_failure;
use crate::physical_runtime::record_serving::{
    RecordAppendBatch, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    PhysicalMutationPreparationOutcome, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, PhysicalMutationResourceShape, PreparedPhysicalMutation,
    PreparedPhysicalMutationContext,
};

const SOURCE_LIMIT: u64 = 256 * 1024;

/// One current inline segment file selected for its own atomic rewrite.
#[derive(Debug, Clone, Copy)]
pub struct PlannedInlineRewriteArtifact {
    segment_id: u64,
    generation: u64,
    pages: u32,
    source_bytes: u64,
    anchor: DurableInlineRecordPlacement,
}

impl PlannedInlineRewriteArtifact {
    pub const fn segment_id(self) -> u64 {
        self.segment_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn pages(self) -> u32 {
        self.pages
    }

    pub const fn source_bytes(self) -> u64 {
        self.source_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineArtifactRewritePlanDenial {
    PublicationAuthorityReleased,
    PhysicalPressure,
    PublishedLayoutDamaged,
}

struct AccumulatedArtifact {
    anchor: DurableInlineRecordPlacement,
    pages: BTreeSet<u64>,
}

impl RecordPublicationDirector {
    /// Select `count` current inline segment artifacts before any rewrite effect.
    ///
    /// One redo window is one source range. A scope that names more than one
    /// artifact is partitioned here, before admission, into one atomic rewrite
    /// per artifact. The selected set and each artifact stay within 256 KiB.
    pub(super) fn plan_inline_artifact_rewrites(
        &self,
        count: u32,
    ) -> Result<Vec<PlannedInlineRewriteArtifact>, InlineArtifactRewritePlanDenial> {
        if count == 0 {
            return Err(InlineArtifactRewritePlanDenial::PublishedLayoutDamaged);
        }
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let (current_root, _) = self.root_owner.snapshot();
        let allocation = self
            .residency
            .begin_operation(
                PhysicalOperationAllocationScope::ForegroundRead,
                NonZeroU64::new(64 * 1024).expect("read grant is nonzero"),
            )
            .map_err(|_| InlineArtifactRewritePlanDenial::PhysicalPressure)?;
        let manifest = ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            current_root.clone(),
        );
        let mut cursor = ManifestRangeCursor::new(manifest);
        cursor
            .seek(&allocation, current_root.routing_root(), None)
            .map_err(manifest_lookup_failure)
            .map_err(plan_denial)?;
        let mut artifacts: BTreeMap<(u64, u64), AccumulatedArtifact> = BTreeMap::new();
        let mut seen = 0_u64;
        let limit = current_root.record_count().saturating_add(1);
        while seen < limit {
            let Some(placement) = cursor
                .next(&allocation)
                .map_err(manifest_lookup_failure)
                .map_err(plan_denial)?
            else {
                break;
            };
            seen = seen.saturating_add(1);
            let CurrentPhysicalRecordPlacement::Inline(placement) = placement else {
                continue;
            };
            let key = (placement.segment().get(), placement.segment_generation());
            let page = placement.page().get();
            artifacts
                .entry(key)
                .and_modify(|artifact| {
                    if placement.page().get() >= artifact.anchor.page().get() {
                        artifact.anchor = placement;
                    }
                    artifact.pages.insert(page);
                })
                .or_insert_with(|| {
                    let mut pages = BTreeSet::new();
                    pages.insert(page);
                    AccumulatedArtifact {
                        anchor: placement,
                        pages,
                    }
                });
        }
        let mut selected = Vec::new();
        let mut total = 0_u64;
        for ((segment_id, generation), artifact) in artifacts {
            if selected.len() == count as usize {
                break;
            }
            let pages = u32::try_from(artifact.pages.len())
                .map_err(|_| InlineArtifactRewritePlanDenial::PublishedLayoutDamaged)?;
            let source_bytes = page_bytes.saturating_mul(u64::from(pages));
            if source_bytes == 0 || source_bytes > SOURCE_LIMIT {
                return Err(InlineArtifactRewritePlanDenial::PhysicalPressure);
            }
            total = total.saturating_add(source_bytes);
            if total > SOURCE_LIMIT {
                return Err(InlineArtifactRewritePlanDenial::PhysicalPressure);
            }
            selected.push(PlannedInlineRewriteArtifact {
                segment_id,
                generation,
                pages,
                source_bytes,
                anchor: artifact.anchor,
            });
        }
        if selected.len() != count as usize {
            return Err(InlineArtifactRewritePlanDenial::PublishedLayoutDamaged);
        }
        Ok(selected)
    }

    pub(super) fn prepare_planned_inline_artifact(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        artifact: PlannedInlineRewriteArtifact,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(outcome) = self.require_preparation_health() {
            return outcome;
        }
        if artifact.pages == 0 || artifact.source_bytes > SOURCE_LIMIT {
            return map_record_denial(RecordAppendDenial::PhysicalPressure);
        }
        if !placement.admits(self.format) {
            return map_record_denial(RecordAppendDenial::PlacementFormatMismatch);
        }
        if artifact.anchor.segment().get() != artifact.segment_id
            || artifact.anchor.segment_generation() != artifact.generation
        {
            return map_record_denial(RecordAppendDenial::PublishedLayoutDamaged);
        }
        let (root, _) = self.root_owner.snapshot();
        let digest = artifact_digest(&artifact);
        let group_queue_admission = match self.group_queue_admission_tick() {
            Ok(tick) => tick,
            Err(outcome) => return outcome,
        };
        let admitted = match self.admit_mutation_preparation(
            placement,
            crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            digest,
            request,
            PhysicalMutationOperationFamily::SegmentRewrite,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return outcome,
        };
        let PhysicalMutationPreparationAdmission::Prepared(admitted) = admitted else {
            return admitted_terminal(admitted);
        };
        let batch = RecordAppendBatch::from_prepared_record_bytes(vec![vec![0]]);
        TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
            PreparedPhysicalMutation::new(
                admitted.admission,
                batch,
                super::super::durable_preparation::CanonicalPayloadMaterializationObservation::default(),
                PreparedPhysicalMutationContext {
                    placement,
                    manifest_capacity_transition:
                        crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
                    deadline: admitted.deadline,
                    group_queue_admission,
                    signal_profile: self.signal_profile,
                    durability_policy_basis: self.durability_policy_basis.clone(),
                    resources: PhysicalMutationResourceShape::prepared(1, artifact.source_bytes),
                    start: crate::physical_runtime::PhysicalMutationRuntimeOwner::start_port(
                        &self.mutations,
                    ),
                    selected_segment_rewrite: false,
                    rewrite_pages: 0,
                    source_root_generation: 0,
                    rewrite_anchor: None,
                },
            )
            .mark_selected_segment_rewrite(
                root.generation(),
                artifact.pages,
                Some(artifact.anchor),
            ),
        ))
        .into()
    }
}

fn artifact_digest(artifact: &PlannedInlineRewriteArtifact) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"store.physical.rewrite-artifact.v1");
    digest.update(artifact.segment_id.to_le_bytes());
    digest.update(artifact.generation.to_le_bytes());
    digest.update(artifact.pages.to_le_bytes());
    digest.update(artifact.anchor.record().allocation_epoch());
    digest.update(artifact.anchor.record().ordinal().to_le_bytes());
    digest.finalize().into()
}

fn plan_denial(error: RecordAppendError) -> InlineArtifactRewritePlanDenial {
    match error {
        RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure)
        | RecordAppendError::PhysicalPressure { .. } => {
            InlineArtifactRewritePlanDenial::PhysicalPressure
        }
        RecordAppendError::Denied(_) | RecordAppendError::StreamFailed(_) => {
            InlineArtifactRewritePlanDenial::PublishedLayoutDamaged
        }
    }
}
