use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use worth_relational::facade::identity::KindId;

use super::WorthQueryGraphCompositionBreadth;
use crate::runtime::WorthQueryMutationTargetCollectionIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphCompositionProgramStepKind {
    SymbolicEntityDeclaration,
    SymbolicEntityFollowupMutation,
    RelationDeclaration,
    SymbolicRelationDeclaration,
    SymbolicRelationFollowupMutation,
    SymbolicRelationRetirement,
    ExistingTargetFollowupMutation,
    ExistingTargetRetarget,
    ExistingTargetSupersession,
    ExistingTargetRetirement,
    ExistingTargetVerifiedFollowupMutation,
    ExistingTargetVerifiedRetarget,
    ExistingTargetVerifiedSupersession,
    ExistingTargetVerifiedRetirement,
}

impl WorthQueryGraphCompositionProgramStepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SymbolicEntityDeclaration => "symbolic-entity-declaration",
            Self::SymbolicEntityFollowupMutation => "symbolic-entity-followup-mutation",
            Self::RelationDeclaration => "relation-declaration",
            Self::SymbolicRelationDeclaration => "symbolic-relation-declaration",
            Self::SymbolicRelationFollowupMutation => "symbolic-relation-followup-mutation",
            Self::SymbolicRelationRetirement => "symbolic-relation-retirement",
            Self::ExistingTargetFollowupMutation => "existing-target-followup-mutation",
            Self::ExistingTargetRetarget => "existing-target-retarget",
            Self::ExistingTargetSupersession => "existing-target-supersession",
            Self::ExistingTargetRetirement => "existing-target-retirement",
            Self::ExistingTargetVerifiedFollowupMutation => {
                "existing-target-verified-followup-mutation"
            }
            Self::ExistingTargetVerifiedRetarget => "existing-target-verified-retarget",
            Self::ExistingTargetVerifiedSupersession => "existing-target-verified-supersession",
            Self::ExistingTargetVerifiedRetirement => "existing-target-verified-retirement",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionProgramStep {
    component_index: usize,
    kind: WorthQueryGraphCompositionProgramStepKind,
    declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    relation_kind_id: Option<KindId>,
    declared_symbol: Option<String>,
}

impl WorthQueryGraphCompositionProgramStep {
    pub(crate) fn new(
        component_index: usize,
        kind: WorthQueryGraphCompositionProgramStepKind,
        declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
        declared_symbol: Option<String>,
    ) -> Self {
        Self {
            component_index,
            kind,
            declared_collection,
            relation_kind_id: None,
            declared_symbol,
        }
    }

    pub(crate) fn with_relation_kind_id(mut self, relation_kind_id: KindId) -> Self {
        self.relation_kind_id = Some(relation_kind_id);
        self
    }

    pub fn component_index(&self) -> usize {
        self.component_index
    }

    pub fn kind(&self) -> WorthQueryGraphCompositionProgramStepKind {
        self.kind
    }

    pub fn declared_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.declared_collection.as_ref()
    }

    pub fn declared_collection(&self) -> &str {
        self.declared_collection
            .as_ref()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
            .unwrap_or("")
    }

    pub fn relation_kind_id(&self) -> Option<KindId> {
        self.relation_kind_id
    }

    pub fn declared_symbol(&self) -> Option<&str> {
        self.declared_symbol.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionProgram {
    steps: Vec<WorthQueryGraphCompositionProgramStep>,
    program_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionProgram {
    pub(crate) fn new(
        steps: Vec<WorthQueryGraphCompositionProgramStep>,
        breadth: &WorthQueryGraphCompositionBreadth,
    ) -> Result<Self, IdentityDerivationDenial> {
        let step_digests = steps
            .iter()
            .map(|step| {
                worth_query_evidence_identity(
                    WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-program-step",
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("component"),
                    step.component_index(),
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), step.kind().as_str())
                .field_value(
                    WorthQueryEvidenceTag::new("declared_collection"),
                    step.declared_collection(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("relation_kind_id"),
                    step.relation_kind_id()
                        .map(|kind_id| kind_id.0.to_string())
                        .as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("declared_symbol"),
                    step.declared_symbol(),
                )
                .seal()
            })
            .collect::<Vec<_>>();
        let program_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-program",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("breadth"),
                    breadth.breadth_evidence_digest(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("step"),
                    step_digests.iter(),
                )
                .seal()?;
        Ok(Self {
            steps,
            program_digest,
        })
    }

    pub(crate) fn empty() -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            Vec::new(),
            &WorthQueryGraphCompositionBreadth::empty()?,
        )?)
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn component_count(&self) -> usize {
        self.steps.len()
    }

    pub fn steps(&self) -> &[WorthQueryGraphCompositionProgramStep] {
        &self.steps
    }

    pub fn program_digest(&self) -> &str {
        self.program_digest.as_str()
    }

    pub fn program_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.program_digest
    }
}
