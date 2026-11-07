use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope};
use crate::runtime::tests::support::*;
use crate::{
    authoring::{
        AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
        RawAuthoredResultShape, RootEntityKey,
    },
    authorized_projection::{
        derive_authorized_projection, AuthorizedProjectionArtifact, PolicyAspectMask,
        PolicyInfluenceSet,
    },
    canonicalization::CanonicalResultShapeArtifact,
    projection_consumption::ProjectionAuthorityContract,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime::tests) struct RuntimeHostileCertificationCounters {
    committed_read_hot_path_lock_count: usize,
    shared_read_mint_row_clone_count: usize,
    published_artifact_registry_lease_count: usize,
    reader_derived_evaluation_count: usize,
    orphaned_snapshot_generation_count: usize,
    unretired_read_pin_count: usize,
    journal_gap_count: usize,
    delivery_residue_count: usize,
    digest: WorthQueryEvidenceIdentity,
}

impl RuntimeHostileCertificationCounters {
    pub(in crate::runtime::tests) fn for_runtime(
        runtime: &WorthQueryRuntime,
        journal_gap_count: usize,
        reader_derived_evaluation_count: usize,
        delivery_residue_count: usize,
    ) -> Self {
        let committed_read_hot_path_lock_count =
            shared_read_structural_lock_acquisition_count(runtime);
        let shared_read_mint_row_clone_count = shared_read_mint_row_clone_count(runtime);
        let published_artifact_registry_lease_count =
            published_artifact_registry_lease_count(runtime);
        let orphaned_snapshot_generation_count =
            shared_read_structural_orphaned_generation_count(runtime);
        let unretired_read_pin_count = shared_read_structural_unretired_pin_count(runtime);
        let digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("committed_read_hot_path_lock_count"),
            committed_read_hot_path_lock_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("shared_read_mint_row_clone_count"),
            shared_read_mint_row_clone_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("published_artifact_registry_lease_count"),
            published_artifact_registry_lease_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("reader_derived_evaluation_count"),
            reader_derived_evaluation_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("orphaned_snapshot_generation_count"),
            orphaned_snapshot_generation_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("unretired_read_pin_count"),
            unretired_read_pin_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("journal_gap_count"),
            journal_gap_count,
        )
        .field_usize(
            crate::WorthQueryEvidenceTag::new("delivery_residue_count"),
            delivery_residue_count,
        )
        .seal()
        .expect("fixture digest must fit");
        Self {
            committed_read_hot_path_lock_count,
            shared_read_mint_row_clone_count,
            published_artifact_registry_lease_count,
            reader_derived_evaluation_count,
            orphaned_snapshot_generation_count,
            unretired_read_pin_count,
            journal_gap_count,
            delivery_residue_count,
            digest,
        }
    }

    pub(in crate::runtime::tests) fn committed_read_hot_path_lock_count(&self) -> usize {
        self.committed_read_hot_path_lock_count
    }

    pub(in crate::runtime::tests) fn shared_read_mint_row_clone_count(&self) -> usize {
        self.shared_read_mint_row_clone_count
    }

    pub(in crate::runtime::tests) fn published_artifact_registry_lease_count(&self) -> usize {
        self.published_artifact_registry_lease_count
    }

    pub(in crate::runtime::tests) fn reader_derived_evaluation_count(&self) -> usize {
        self.reader_derived_evaluation_count
    }

    pub(in crate::runtime::tests) fn orphaned_snapshot_generation_count(&self) -> usize {
        self.orphaned_snapshot_generation_count
    }

    pub(in crate::runtime::tests) fn unretired_read_pin_count(&self) -> usize {
        self.unretired_read_pin_count
    }

    pub(in crate::runtime::tests) fn journal_gap_count(&self) -> usize {
        self.journal_gap_count
    }

    pub(in crate::runtime::tests) fn delivery_residue_count(&self) -> usize {
        self.delivery_residue_count
    }

    pub(in crate::runtime::tests) fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }
}

fn shared_read_structural_lock_acquisition_count(_runtime: &WorthQueryRuntime) -> usize {
    _runtime
        .shared_read_counters()
        .committed_read_hot_path_lock_count()
}

fn shared_read_mint_row_clone_count(_runtime: &WorthQueryRuntime) -> usize {
    _runtime
        .shared_read_counters()
        .shared_read_mint_row_clone_count()
}

fn published_artifact_registry_lease_count(_runtime: &WorthQueryRuntime) -> usize {
    _runtime
        .shared_read_counters()
        .published_artifact_registry_lease_count()
}

fn shared_read_structural_orphaned_generation_count(_runtime: &WorthQueryRuntime) -> usize {
    _runtime.shared_read_counters().orphaned_generation_count()
}

fn shared_read_structural_unretired_pin_count(_runtime: &WorthQueryRuntime) -> usize {
    _runtime.shared_read_counters().unretired_pin_count()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime::tests) struct RuntimeHostileCertificationArtifact {
    receipt_digests: Vec<String>,
    reader_result_digests: Vec<String>,
    published_artifact_digests: Vec<String>,
    preview_closeout_digests: Vec<String>,
    branch_basis_digests: Vec<String>,
    counters: RuntimeHostileCertificationCounters,
    digest: WorthQueryEvidenceIdentity,
}

impl RuntimeHostileCertificationArtifact {
    pub(in crate::runtime::tests) fn new(
        receipt_digests: Vec<String>,
        reader_result_digests: Vec<String>,
        published_artifact_digests: Vec<String>,
        preview_closeout_digests: Vec<String>,
        branch_basis_digests: Vec<String>,
        counters: RuntimeHostileCertificationCounters,
    ) -> Self {
        let digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("receipt_digest"),
            receipt_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("reader_result_digest"),
            reader_result_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("published_artifact_digest"),
            published_artifact_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("preview_closeout_digest"),
            preview_closeout_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("branch_basis_digest"),
            branch_basis_digests.iter().map(String::as_str),
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("counter_digest"),
            counters.digest().terminal_projection_for_reporting(),
        )
        .seal()
        .expect("fixture digest must fit");
        Self {
            receipt_digests,
            reader_result_digests,
            published_artifact_digests,
            preview_closeout_digests,
            branch_basis_digests,
            counters,
            digest,
        }
    }

    pub(in crate::runtime::tests) fn counters(&self) -> &RuntimeHostileCertificationCounters {
        &self.counters
    }

    pub(in crate::runtime::tests) fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }
}

pub(in crate::runtime::tests) fn hostile_write_receipt_digest(
    receipt: &WorthQueryWriteReceipt,
) -> String {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("mutation_family"),
        receipt.mutation_family().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("authority_lane"),
        receipt.authority_lane().as_str(),
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("commit_identity"),
        receipt.commit_evidence_identity(),
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("snapshot_identity"),
        receipt.snapshot_evidence_identity(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("declared_aspect_value_digest"),
        receipt.declared_aspect_value_digest().unwrap_or("none"),
    )
    .seal()
    .expect("fixture digest must fit")
    .terminal_projection_for_reporting()
    .to_string()
}

pub(in crate::runtime::tests) fn hostile_published_artifact_digest(
    artifact: &WorthQueryPublishedDerivedArtifactHandle,
    consumed_title: Option<&str>,
) -> String {
    let inspection = artifact.inspect_projection_consumption();
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("snapshot_identity"),
        &artifact.snapshot_identity().evidence_identity(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("view_name"),
        artifact.view_name(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("published"),
        if inspection.published() {
            "true"
        } else {
            "false"
        },
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("binding_digest"),
        inspection
            .artifact_binding_for_reporting()
            .unwrap_or("none"),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("async_result_state_digest"),
        inspection
            .async_result_state()
            .map(|state| state.result_state_for_reporting())
            .unwrap_or("none"),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("consumed_title"),
        consumed_title.unwrap_or("none"),
    )
    .seal()
    .expect("fixture digest must fit")
    .terminal_projection_for_reporting()
    .to_string()
}

pub(in crate::runtime::tests) fn hostile_preview_closeout_digest(
    outcome: &WorthQueryPreviewOutcome,
) -> String {
    outcome.closeout_evidence().closeout_digest().to_string()
}

pub(in crate::runtime::tests) fn hostile_preview_delivery_residue_count(
    outcome: &WorthQueryPreviewOutcome,
) -> usize {
    outcome.effect_delivery_residue_count()
}

pub(in crate::runtime::tests) fn hostile_branch_basis_digest(
    session: &WorthQueryBranchSession<'_>,
) -> String {
    session
        .basis_admission()
        .admission_digest()
        .terminal_projection_for_reporting()
        .to_string()
}

pub(in crate::runtime::tests) fn hostile_journal_gap_count(
    receipts: &[WorthQueryWriteReceipt],
) -> usize {
    let ordinals = receipts
        .iter()
        .map(|receipt| receipt.journal_position().ordinal_for_reporting())
        .collect::<Vec<_>>();
    ordinals
        .windows(2)
        .filter(|window| matches!(window, [left, right] if *right != *left + 1))
        .count()
}

pub(in crate::runtime::tests) fn hostile_insert_task_command(
    id: &str,
    title: &str,
) -> WorthQueryWriteCommand {
    WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("identity.id"),
            test_authored_string_aspect_value(id),
        )
        .set_aspect(
            test_aspect_touch("title.value"),
            test_authored_string_aspect_value(title),
        )
        .build_insert("Task")
        .expect("insert command should build")
}

pub(in crate::runtime::tests) fn hostile_consume_title_attempt(
    artifact: &WorthQueryPublishedDerivedArtifactHandle,
) -> WorthQueryPublishedProjectionAuthorityOutcome {
    let (result_shape, authorized_projection) = hostile_projection_artifacts();
    artifact
        .consume_projection_authority(
            &result_shape,
            &authorized_projection,
            ProjectionAuthorityContract::declare()
                .require_settled_consumption()
                .require_source_authority()
                .require_display_field(
                    crate::projection_consumption::projection_fact_field_path_from_segments([
                        worth_foundational::facade::FieldKey::new("title")
                            .expect("projection fact field segment should admit"),
                        worth_foundational::facade::FieldKey::new("value")
                            .expect("projection fact field segment should admit"),
                    ]),
                ),
        )
        .expect("projection consumption should remain on the typed artifact lane")
}

#[path = "hostile_certification/hostile_projection_artifacts.rs"]
mod hostile_projection_artifacts;
pub use hostile_projection_artifacts::hostile_projection_artifacts;
