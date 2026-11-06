use super::WorthQueryGraphCompositionProgram;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryGraphCompositionProgramStepKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphCompositionLifecycleOutcomeKind {
    Created,
    UpdatedIdentityPreserved,
    RetargetedIdentityPreserved,
    RetiredCurrentTruth,
    SupersededWithLineage,
    DeletedIfUncommitted,
    DeniedBeforeExecution,
}

impl WorthQueryGraphCompositionLifecycleOutcomeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::UpdatedIdentityPreserved => "updated-identity-preserved",
            Self::RetargetedIdentityPreserved => "retargeted-identity-preserved",
            Self::RetiredCurrentTruth => "retired-current-truth",
            Self::SupersededWithLineage => "superseded-with-lineage",
            Self::DeletedIfUncommitted => "deleted-if-uncommitted",
            Self::DeniedBeforeExecution => "denied-before-execution",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionLifecycleOutcomeEntry {
    component_index: usize,
    outcome_kind: WorthQueryGraphCompositionLifecycleOutcomeKind,
    declared_collection: String,
    declared_symbol: Option<String>,
}

impl WorthQueryGraphCompositionLifecycleOutcomeEntry {
    fn new(
        component_index: usize,
        outcome_kind: WorthQueryGraphCompositionLifecycleOutcomeKind,
        declared_collection: impl Into<String>,
        declared_symbol: Option<String>,
    ) -> Self {
        Self {
            component_index,
            outcome_kind,
            declared_collection: declared_collection.into(),
            declared_symbol,
        }
    }

    pub fn component_index(&self) -> usize {
        self.component_index
    }

    pub fn outcome_kind(&self) -> WorthQueryGraphCompositionLifecycleOutcomeKind {
        self.outcome_kind
    }

    pub fn declared_collection(&self) -> &str {
        &self.declared_collection
    }

    pub fn declared_symbol(&self) -> Option<&str> {
        self.declared_symbol.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionLifecycleOutcomes {
    entries: Vec<WorthQueryGraphCompositionLifecycleOutcomeEntry>,
    lifecycle_digest: WorthQueryEvidenceIdentity,
    counter_snapshot: String,
}

impl WorthQueryGraphCompositionLifecycleOutcomes {
    pub(crate) fn derive(
        program: &WorthQueryGraphCompositionProgram,
    ) -> Result<Option<Self>, IdentityDerivationDenial> {
        if program.is_empty() {
            return Ok(None);
        }

        let entries = program
            .steps()
            .iter()
            .map(|step| {
                let outcome_kind = match step.kind() {
                    WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
                    | WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
                    | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
                    }
                    WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
                    | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
                    }
                    WorthQueryGraphCompositionProgramStepKind::ExistingTargetFollowupMutation
                    | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedFollowupMutation => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
                    }
                    WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetarget
                    | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetarget => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::RetargetedIdentityPreserved
                    }
                    WorthQueryGraphCompositionProgramStepKind::ExistingTargetSupersession
                    | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::SupersededWithLineage
                    }
                    WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement
                    | WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetirement
                    | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetirement => {
                        WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
                    }
                };
                WorthQueryGraphCompositionLifecycleOutcomeEntry::new(
                    step.component_index(),
                    outcome_kind,
                    step.declared_collection(),
                    step.declared_symbol().map(str::to_string),
                )
            })
            .collect::<Vec<_>>();

        let created_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind() == WorthQueryGraphCompositionLifecycleOutcomeKind::Created
            })
            .count();
        let updated_identity_preserved_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
            })
            .count();
        let retargeted_identity_preserved_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::RetargetedIdentityPreserved
            })
            .count();
        let retired_current_truth_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
            })
            .count();
        let superseded_with_lineage_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::SupersededWithLineage
            })
            .count();
        let deleted_if_uncommitted_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::DeletedIfUncommitted
            })
            .count();
        let denied_before_execution_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_kind()
                    == WorthQueryGraphCompositionLifecycleOutcomeKind::DeniedBeforeExecution
            })
            .count();
        let counter_snapshot = diagnostic_counter_snapshot(&[
            ("created", created_count),
            (
                "updated_identity_preserved",
                updated_identity_preserved_count,
            ),
            (
                "retargeted_identity_preserved",
                retargeted_identity_preserved_count,
            ),
            ("retired_current_truth", retired_current_truth_count),
            ("superseded_with_lineage", superseded_with_lineage_count),
            ("deleted_if_uncommitted", deleted_if_uncommitted_count),
            ("denied_before_execution", denied_before_execution_count),
        ]);
        let lifecycle_entry_digests = entries
            .iter()
            .map(|entry| {
                worth_query_evidence_identity(
                    WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-lifecycle-entry",
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("component"),
                    entry.component_index(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("outcome"),
                    entry.outcome_kind().as_str(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("declared_collection"),
                    entry.declared_collection(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("declared_symbol"),
                    entry.declared_symbol(),
                )
                .seal()
            })
            .collect::<Vec<_>>();
        let lifecycle_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-lifecycle-outcomes",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("program"),
                    program.program_evidence_digest(),
                )
                .field_usize(WorthQueryEvidenceTag::new("created_count"), created_count)
                .field_usize(
                    WorthQueryEvidenceTag::new("updated_identity_preserved_count"),
                    updated_identity_preserved_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("retargeted_identity_preserved_count"),
                    retargeted_identity_preserved_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("retired_current_truth_count"),
                    retired_current_truth_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("superseded_with_lineage_count"),
                    superseded_with_lineage_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("deleted_if_uncommitted_count"),
                    deleted_if_uncommitted_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("denied_before_execution_count"),
                    denied_before_execution_count,
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("entry"),
                    lifecycle_entry_digests.iter(),
                )
                .seal()?;

        Ok(Some(Self {
            entries,
            lifecycle_digest,
            counter_snapshot,
        }))
    }

    pub fn entries(&self) -> &[WorthQueryGraphCompositionLifecycleOutcomeEntry] {
        &self.entries
    }

    pub fn lifecycle_digest(&self) -> &str {
        self.lifecycle_digest.as_str()
    }

    pub fn lifecycle_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.lifecycle_digest
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }
}

fn diagnostic_counter_snapshot(fields: &[(&str, usize)]) -> String {
    let mut snapshot = String::new();
    for (index, (label, value)) in fields.iter().enumerate() {
        if index > 0 {
            snapshot.push(';');
        }
        snapshot.push_str(label);
        snapshot.push('=');
        snapshot.push_str(&value.to_string());
    }
    snapshot
}
