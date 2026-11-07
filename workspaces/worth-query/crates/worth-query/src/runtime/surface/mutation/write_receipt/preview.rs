use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQuerySnapshotIdentity,
};
use crate::runtime::{
    WorthQueryAuthorityLane, WorthQueryExistingTruthBindingEvidence, WorthQueryJournalPosition,
    WorthQueryMutationFamily, WorthQueryMutationTargetCollectionIdentity, WorthQueryWriteReceipt,
};
use crate::session_label::WorthQuerySessionLabel;

use super::{
    helpers::{
        continuity_mutation_evidence, naming_mutation_evidence, symbolic_target_reference_evidence,
        target_evidence_from_receipt,
    },
    WorthQueryWriteCommand,
};

impl WorthQueryWriteReceipt {
    pub(in crate::runtime) fn preview(
        label: &WorthQuerySessionLabel,
        sequence: usize,
        command: &WorthQueryWriteCommand,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let preview_identity = preview_write_receipt_identity(label, sequence)?;
        let commit_identity = WorthQueryCommitIdentity::preview(preview_identity.clone());
        let commit_evidence_identity =
            super::write_receipt_commit_evidence_identity(&commit_identity)?;
        let journal_position =
            WorthQueryJournalPosition::preview(preview_identity.clone(), sequence as u64)?;
        let snapshot_evidence_identity =
            super::write_receipt_snapshot_evidence_identity(&snapshot_identity)?;
        let delta = preview_receipt_delta(command, &preview_identity)?;
        let inner = crate::memory_workspace::WorthQueryMutationReceipt {
            commit_identity,
            snapshot_identity,
            deltas: vec![delta.clone()],
            bridge_authority: None,
        };
        let committed_truth_identity = super::write_receipt_committed_truth_identity(&inner)?;
        let target_entity_identity = preview_target_entity_identity(command, &delta);
        let declared_collection_identity = command.declared_collection_identity();
        let target_collection_identity = declared_collection_identity
            .as_ref()
            .map(|collection| {
                WorthQueryMutationTargetCollectionIdentity::new(
                    "write-receipt-preview-target",
                    collection.as_str(),
                )
            })
            .transpose()?;
        let naming_mutation_evidence = naming_mutation_evidence(
            None,
            command.naming_intent(),
            target_entity_identity
                .as_ref()
                .or(Some(&delta.entity_identity)),
            target_collection_identity.as_ref()?,
        )?;
        let continuity_mutation_evidence = continuity_mutation_evidence(
            None,
            command.continuity_intent(),
            command.existing_truth_binding(),
            target_entity_identity
                .as_ref()
                .or(Some(&delta.entity_identity)),
            target_collection_identity.as_ref()?,
        )?;
        Ok(Self {
            inner,
            commit_evidence_identity,
            committed_truth_identity,
            journal_position,
            snapshot_evidence_identity,
            mutation_family: command.mutation_family(),
            authority_lane: WorthQueryAuthorityLane::PreviewTruth,
            basis_lane: WorthQueryAuthorityLane::PreviewTruth,
            target_evidence: target_evidence_from_receipt(
                command.mutation_family(),
                declared_collection_identity.clone(),
                command.declared_entity_identity(),
                target_collection_identity.clone()?,
                command.declared_entity_identity(),
            ),
            existing_truth_assertion_evidence: None,
            existing_truth_binding_evidence: command
                .existing_truth_binding()
                .map(WorthQueryExistingTruthBindingEvidence::from_binding)
                .transpose()?,
            symbolic_target_reference_evidence: symbolic_target_reference_evidence(
                command.mutation_family(),
                None,
                command.symbolic_target_reference(),
                Some(&delta.entity_identity),
            )?,
            symbolic_aspect_resolution_evidence: Vec::new(),
            naming_mutation_evidence,
            continuity_mutation_evidence,
            causality_evidence: None,
            provenance_evidence: None,
            declared_collection_identity,
            declared_entity_identity: command.declared_entity_identity(),
            target_collection_identity,
            target_entity_identity,
            declared_aspect_operations: command.declared_aspect_operations(),
            declared_aspect_value_digest: crate::runtime::command_declared_aspect_value_identity(
                command,
            )?,
            mutation_metadata: command.mutation_metadata(),
            affected_live_view_targets: Vec::new(),
            affected_derived_view_targets: Vec::new(),
            considered_computed_view_count: 0,
            considered_effect_count: 0,
            delivered_effect_count: 0,
            pending_write_intent_count: 0,
            suppressed_effect_count: 0,
            meaningful_effect_suppression_count: 0,
            effect_expression_failure_count: 0,
            refresh_fallback: false,
            decision_trace_envelope: None,
            execution_provenance: None,
        })
    }
}

pub(super) fn preview_receipt_delta(
    command: &WorthQueryWriteCommand,
    preview_identity: &WorthQueryEvidenceIdentity,
) -> Result<crate::memory_workspace::WorthQueryMutationDelta, IdentityDerivationDenial> {
    Ok(match command {
        WorthQueryWriteCommand::InsertAspects { collection, .. } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                collection.as_str(),
                WorthQueryEntityIdentity::preview(preview_identity.clone()),
                crate::memory_workspace::WorthQueryMutationKind::Created,
                command.declared_aspect_touches(),
            )?
        }
        WorthQueryWriteCommand::UpdateAspect {
            entity_identity,
            aspect,
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            "preview",
            entity_identity.clone(),
            crate::memory_workspace::WorthQueryMutationKind::Updated,
            vec![crate::runtime::WorthQueryAspectTouch::from_parsed_target(
                aspect.parsed_target().clone(),
            )],
        )?,
        WorthQueryWriteCommand::UpdateAspects {
            entity_identity, ..
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            "preview",
            entity_identity.clone(),
            crate::memory_workspace::WorthQueryMutationKind::Updated,
            command.declared_aspect_touches(),
        )?,
        WorthQueryWriteCommand::UpdateExistingAspects { binding, .. } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                binding
                    .terminal_target_collection_projection()
                    .map(str::to_string)
                    .unwrap_or_else(|| "preview".to_string()),
                binding.resolved_entity_artifact_identity(),
                crate::memory_workspace::WorthQueryMutationKind::Updated,
                command.declared_aspect_touches(),
            )?
        }
        WorthQueryWriteCommand::VerifyThenUpdateExistingAspects { binding, .. } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                binding
                    .terminal_target_collection_projection()
                    .map(str::to_string)
                    .unwrap_or_else(|| "preview".to_string()),
                binding.resolved_entity_artifact_identity(),
                crate::memory_workspace::WorthQueryMutationKind::Updated,
                command.declared_aspect_touches(),
            )?
        }
        WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
            binding,
            touched_aspects,
            ..
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            binding
                .terminal_target_collection_projection()
                .map(str::to_string)
                .unwrap_or_else(|| "preview".to_string()),
            binding.resolved_entity_artifact_identity(),
            crate::memory_workspace::WorthQueryMutationKind::Deleted,
            touched_aspects.to_vec(),
        )?,
        WorthQueryWriteCommand::AssertExistingAspects { binding, .. }
        | WorthQueryWriteCommand::VerifyExistingAspects { binding, .. } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                binding
                    .terminal_target_collection_projection()
                    .map(str::to_string)
                    .unwrap_or_else(|| "preview".to_string()),
                binding.resolved_entity_artifact_identity(),
                crate::memory_workspace::WorthQueryMutationKind::Updated,
                command.declared_aspect_touches(),
            )?
        }
        WorthQueryWriteCommand::UpdateSymbolicAspects { reference, .. } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                reference
                    .target_collection()
                    .map(str::to_string)
                    .unwrap_or_else(|| "preview".to_string()),
                preview_symbolic_entity_identity(reference)?,
                crate::memory_workspace::WorthQueryMutationKind::Updated,
                command.declared_aspect_touches(),
            )?
        }
        WorthQueryWriteCommand::DeleteAspects {
            entity_identity,
            declared_collection,
            touched_aspects,
            ..
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            declared_collection
                .as_ref()
                .map(|collection| collection.as_str())
                .unwrap_or("preview"),
            entity_identity.clone(),
            crate::memory_workspace::WorthQueryMutationKind::Deleted,
            touched_aspects.to_vec(),
        )?,
        WorthQueryWriteCommand::DeleteExistingAspects {
            binding,
            touched_aspects,
            ..
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            binding
                .terminal_target_collection_projection()
                .map(str::to_string)
                .unwrap_or_else(|| "preview".to_string()),
            binding.resolved_entity_artifact_identity(),
            crate::memory_workspace::WorthQueryMutationKind::Deleted,
            touched_aspects.to_vec(),
        )?,
        WorthQueryWriteCommand::DeleteSymbolicAspects {
            reference,
            touched_aspects,
            ..
        } => crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            reference
                .target_collection()
                .map(str::to_string)
                .unwrap_or_else(|| "preview".to_string()),
            preview_symbolic_entity_identity(reference)?,
            crate::memory_workspace::WorthQueryMutationKind::Deleted,
            touched_aspects.to_vec(),
        )?,
        WorthQueryWriteCommand::Delete { entity_identity } => {
            crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
                "preview",
                entity_identity.clone(),
                crate::memory_workspace::WorthQueryMutationKind::Deleted,
                Vec::new(),
            )?
        }
    })
}

fn preview_write_receipt_identity(
    label: &WorthQuerySessionLabel,
    sequence: usize,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewWriteReceiptIdentity)
            .field_value(
                WorthQueryEvidenceTag::new("session_label_identity"),
                label.identity_digest().as_str(),
            )
            .field_usize(WorthQueryEvidenceTag::new("sequence"), sequence)
            .seal()?,
    )
}

fn preview_target_entity_identity(
    command: &WorthQueryWriteCommand,
    delta: &crate::memory_workspace::WorthQueryMutationDelta,
) -> Option<WorthQueryEntityIdentity> {
    match command.mutation_family() {
        WorthQueryMutationFamily::Insert => Some(delta.entity_identity.clone()),
        _ => command.declared_entity_identity(),
    }
}

fn preview_symbolic_entity_identity(
    reference: &crate::runtime::WorthQuerySymbolicTargetReference,
) -> Result<WorthQueryEntityIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryEntityIdentity::preview(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewWriteReceiptIdentity)
            .field_value(WorthQueryEvidenceTag::new("symbol"), reference.symbol())
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_collection"),
                reference
                    .target_collection_identity()
                    .map(|collection| collection.evidence_identity()),
            )
            .seal()?,
    ))
}
