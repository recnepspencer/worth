use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAspectTouch,
    WorthQueryGraphCompositionProgramStepKind, WorthQueryMutationFamily,
    WorthQueryMutationTargetCollectionIdentity,
};
use worth_relational::facade::identity::KindId;

use super::super::lifecycle_family::WorthQueryGraphTouchLifecycleFamily;
use super::super::read_verb::WorthQueryGraphTouchReadVerb;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphTouchDescriptorRow {
    component_index: usize,
    mutation_family: WorthQueryMutationFamily,
    read_verb: Option<WorthQueryGraphTouchReadVerb>,
    program_step_kind: Option<WorthQueryGraphCompositionProgramStepKind>,
    lifecycle_family: Option<WorthQueryGraphTouchLifecycleFamily>,
    declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    relation_kind_id: Option<KindId>,
    declared_symbol: Option<String>,
    declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
    touched_aspects: Vec<WorthQueryAspectTouch>,
    has_symbolic_target_reference: bool,
    has_existing_truth_binding: bool,
    symbolic_aspect_reference_count: usize,
    row_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphTouchDescriptorRow {
    pub(crate) fn new(
        input: WorthQueryGraphTouchDescriptorRowInput,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::GraphTouchDescriptorRow)
                .field_shape(WorthQueryEvidenceTag::new("role"), "graph-touch-row")
                .field_usize(
                    WorthQueryEvidenceTag::new("component"),
                    input.component_index,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("mutation_family"),
                    input.mutation_family.as_str(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("read_verb"),
                    input.read_verb.map(|verb| verb.as_str()),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("program_step_kind"),
                    input.program_step_kind.map(|kind| kind.as_str()),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("lifecycle_family"),
                    input.lifecycle_family.map(|family| family.as_str()),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("declared_collection"),
                    input
                        .declared_collection
                        .as_ref()
                        .map(WorthQueryMutationTargetCollectionIdentity::as_str),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("relation_kind_id"),
                    input
                        .relation_kind_id
                        .map(|kind_id| kind_id.0.to_string())
                        .as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("declared_symbol"),
                    input.declared_symbol.as_deref(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("declared_aspect_operation"),
                    input
                        .declared_aspect_operations
                        .iter()
                        .map(terminal_declared_aspect_operation_digest_part),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("touched_aspect"),
                    input
                        .touched_aspects
                        .iter()
                        .map(WorthQueryAspectTouch::admitted_touch_digest_part),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("has_symbolic_target_reference"),
                    input.has_symbolic_target_reference,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("has_existing_truth_binding"),
                    input.has_existing_truth_binding,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("symbolic_aspect_reference_count"),
                    input.symbolic_aspect_reference_count,
                )
                .seal()?;
        Ok(Self {
            component_index: input.component_index,
            mutation_family: input.mutation_family,
            read_verb: input.read_verb,
            program_step_kind: input.program_step_kind,
            lifecycle_family: input.lifecycle_family,
            declared_collection: input.declared_collection,
            relation_kind_id: input.relation_kind_id,
            declared_symbol: input.declared_symbol,
            declared_aspect_operations: input.declared_aspect_operations,
            touched_aspects: input.touched_aspects,
            has_symbolic_target_reference: input.has_symbolic_target_reference,
            has_existing_truth_binding: input.has_existing_truth_binding,
            symbolic_aspect_reference_count: input.symbolic_aspect_reference_count,
            row_digest,
        })
    }

    pub fn component_index(&self) -> usize {
        self.component_index
    }

    pub fn mutation_family(&self) -> WorthQueryMutationFamily {
        self.mutation_family
    }

    pub fn read_verb(&self) -> Option<WorthQueryGraphTouchReadVerb> {
        self.read_verb
    }

    pub fn program_step_kind(&self) -> Option<WorthQueryGraphCompositionProgramStepKind> {
        self.program_step_kind
    }

    pub fn lifecycle_family(&self) -> Option<WorthQueryGraphTouchLifecycleFamily> {
        self.lifecycle_family
    }

    pub fn declared_collection(&self) -> Option<&str> {
        self.declared_collection
            .as_ref()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn declared_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.declared_collection.as_ref()
    }

    pub(crate) fn touches_declared_collection(
        &self,
        collection: &WorthQueryMutationTargetCollectionIdentity,
    ) -> bool {
        self.declared_collection
            .as_ref()
            .is_some_and(|declared| declared.same_target_collection_as(collection))
    }

    pub fn relation_kind_id(&self) -> Option<KindId> {
        self.relation_kind_id
    }

    pub fn declared_symbol(&self) -> Option<&str> {
        self.declared_symbol.as_deref()
    }

    pub fn declared_aspect_operations(&self) -> &[WorthQueryAspectMutationOperation] {
        &self.declared_aspect_operations
    }

    pub fn admitted_touched_aspects(&self) -> &[WorthQueryAspectTouch] {
        &self.touched_aspects
    }

    pub fn has_symbolic_target_reference(&self) -> bool {
        self.has_symbolic_target_reference
    }

    pub fn has_existing_truth_binding(&self) -> bool {
        self.has_existing_truth_binding
    }

    pub fn symbolic_aspect_reference_count(&self) -> usize {
        self.symbolic_aspect_reference_count
    }

    pub fn row_digest(&self) -> &str {
        self.row_digest.as_str()
    }

    pub fn row_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_digest
    }
}

pub(crate) struct WorthQueryGraphTouchDescriptorRowInput {
    pub component_index: usize,
    pub mutation_family: WorthQueryMutationFamily,
    pub read_verb: Option<WorthQueryGraphTouchReadVerb>,
    pub program_step_kind: Option<WorthQueryGraphCompositionProgramStepKind>,
    pub lifecycle_family: Option<WorthQueryGraphTouchLifecycleFamily>,
    pub declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    pub relation_kind_id: Option<KindId>,
    pub declared_symbol: Option<String>,
    pub declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
    pub touched_aspects: Vec<WorthQueryAspectTouch>,
    pub has_symbolic_target_reference: bool,
    pub has_existing_truth_binding: bool,
    pub symbolic_aspect_reference_count: usize,
}

pub(super) fn terminal_declared_aspect_operation_digest_part(
    operation: &WorthQueryAspectMutationOperation,
) -> String {
    format!(
        "{}:{}",
        operation.kind().as_str(),
        operation.aspect_touch().admitted_touch_digest_part()
    )
}
