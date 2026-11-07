use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAspectTouch, WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryWriteCommand,
};
use worth_relational::facade::identity::KindId;

use super::denial::WorthQueryGraphTouchDescriptorDenial;
use super::descriptor_inventory::WorthQueryGraphTouchDescriptorInventory;
use super::descriptor_kind::WorthQueryGraphTouchDescriptorKind;
use super::lifecycle_family::WorthQueryGraphTouchLifecycleFamily;
use super::read_verb::WorthQueryGraphTouchReadVerb;
use super::touch_rows::{
    derive_command_touch_rows, derive_read_touch_rows, WorthQueryGraphReadTouchShape,
    WorthQueryGraphTouchDescriptorRow, WorthQueryGraphTouchDescriptorRowInput,
};
use super::validation::validate_graph_touch_descriptor_inputs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphTouchDescriptor {
    kind: WorthQueryGraphTouchDescriptorKind,
    rows: Vec<WorthQueryGraphTouchDescriptorRow>,
    component_count: usize,
    symbolic_entity_declaration_count: usize,
    symbolic_relation_declaration_count: usize,
    insert_command_count: usize,
    update_command_count: usize,
    assertion_command_count: usize,
    delete_command_count: usize,
    declared_collection_count: usize,
    relation_kind_count: usize,
    declared_aspect_touch_count: usize,
    declared_aspect_operation_count: usize,
    touched_aspect_count: usize,
    descriptor_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphTouchDescriptor {
    pub(crate) fn from_authoritative_mutation_batch(
        program: &WorthQueryGraphCompositionProgram,
        breadth: &WorthQueryGraphCompositionBreadth,
        commands: &[WorthQueryWriteCommand],
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        Self::derive(
            WorthQueryGraphTouchDescriptorKind::AuthoritativeMutationBatch,
            program,
            breadth,
            commands,
        )
    }

    pub fn read_family(
        collection: impl Into<String>,
        verbs: impl IntoIterator<Item = WorthQueryGraphTouchReadVerb>,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        Self::from_read_rows(
            WorthQueryGraphTouchDescriptorKind::ReadFamily,
            collection,
            verbs,
            WorthQueryGraphReadTouchShape::default(),
        )
    }

    pub fn read_family_shape(
        collection: impl Into<String>,
        verbs: impl IntoIterator<Item = WorthQueryGraphTouchReadVerb>,
        shape: WorthQueryGraphReadTouchShape,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        Self::from_read_rows(
            WorthQueryGraphTouchDescriptorKind::ReadFamily,
            collection,
            verbs,
            shape,
        )
    }

    pub fn live_read(
        collection: impl Into<String>,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        Self::from_read_rows(
            WorthQueryGraphTouchDescriptorKind::LiveRead,
            collection,
            [WorthQueryGraphTouchReadVerb::RetainsLiveSubscription],
            WorthQueryGraphReadTouchShape::default(),
        )
    }

    pub fn live_read_shape(
        collection: impl Into<String>,
        verbs: impl IntoIterator<Item = WorthQueryGraphTouchReadVerb>,
        shape: WorthQueryGraphReadTouchShape,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        Self::from_read_rows(
            WorthQueryGraphTouchDescriptorKind::LiveRead,
            collection,
            verbs,
            shape,
        )
    }

    pub fn declared_mutation_collection(
        collection: impl Into<String>,
        mutation_family: crate::runtime::WorthQueryMutationFamily,
        lifecycle_family: Option<WorthQueryGraphTouchLifecycleFamily>,
        declared_aspect_operations: impl IntoIterator<Item = WorthQueryAspectMutationOperation>,
        touched_aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        let collection = collection.into().trim().to_string();
        if collection.is_empty() {
            return Err(WorthQueryGraphTouchDescriptorDenial::new(
                super::denial::WorthQueryGraphTouchDescriptorDenialKind::EmptyDeclaredMutationCollection,
                "declared mutation graph touch descriptor requires a collection",
            ));
        }
        let row = WorthQueryGraphTouchDescriptorRow::new(WorthQueryGraphTouchDescriptorRowInput {
            component_index: 0,
            mutation_family,
            read_verb: None,
            program_step_kind: None,
            lifecycle_family,
            declared_collection: Some(WorthQueryMutationTargetCollectionIdentity::new(
                "graph-touch-descriptor-declared",
                collection,
            )?),
            relation_kind_id: None,
            declared_symbol: None,
            declared_aspect_operations: sorted_unique_operations(declared_aspect_operations),
            touched_aspects: sorted_unique_touches(touched_aspects),
            has_symbolic_target_reference: false,
            has_existing_truth_binding: false,
            symbolic_aspect_reference_count: 0,
        })?;
        let breadth = WorthQueryGraphCompositionBreadth::empty();
        let program = WorthQueryGraphCompositionProgram::empty()?;
        Self::from_rows(
            WorthQueryGraphTouchDescriptorKind::AuthoritativeMutationBatch,
            vec![row],
            &breadth,
            &program,
            1,
        )
    }

    fn derive(
        kind: WorthQueryGraphTouchDescriptorKind,
        program: &WorthQueryGraphCompositionProgram,
        breadth: &WorthQueryGraphCompositionBreadth,
        commands: &[WorthQueryWriteCommand],
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        validate_graph_touch_descriptor_inputs(program, breadth, commands)?;
        let rows = derive_command_touch_rows(program, commands)?;
        Self::from_rows(kind, rows, breadth, program, commands.len())
    }

    fn from_read_rows(
        kind: WorthQueryGraphTouchDescriptorKind,
        collection: impl Into<String>,
        verbs: impl IntoIterator<Item = WorthQueryGraphTouchReadVerb>,
        shape: WorthQueryGraphReadTouchShape,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        let collection = collection.into().trim().to_string();
        if collection.is_empty() {
            return Err(WorthQueryGraphTouchDescriptorDenial::new(
                super::denial::WorthQueryGraphTouchDescriptorDenialKind::EmptyReadCollection,
                "read graph touch descriptor requires a collection",
            ));
        }
        let rows = derive_read_touch_rows(&collection, verbs, &shape)?;
        let breadth = WorthQueryGraphCompositionBreadth::empty();
        let program = WorthQueryGraphCompositionProgram::empty()?;
        Self::from_rows(kind, rows, &breadth, &program, 0)
    }

    fn from_rows(
        kind: WorthQueryGraphTouchDescriptorKind,
        rows: Vec<WorthQueryGraphTouchDescriptorRow>,
        breadth: &WorthQueryGraphCompositionBreadth,
        program: &WorthQueryGraphCompositionProgram,
        command_count: usize,
    ) -> Result<Self, WorthQueryGraphTouchDescriptorDenial> {
        let inventory = WorthQueryGraphTouchDescriptorInventory::from_rows(&rows);
        let row_digests = rows
            .iter()
            .map(WorthQueryGraphTouchDescriptorRow::row_evidence_digest)
            .collect::<Vec<_>>();
        let descriptor_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::GraphTouchDescriptor)
                .field_shape(WorthQueryEvidenceTag::new("role"), "graph-touch-descriptor")
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("graph_breadth"),
                    breadth.breadth_evidence_digest(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("graph_program"),
                    program.program_evidence_digest(),
                )
                .field_usize(WorthQueryEvidenceTag::new("command_count"), command_count)
                .field_usize(
                    WorthQueryEvidenceTag::new("insert_command_count"),
                    inventory.insert_command_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("update_command_count"),
                    inventory.update_command_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("assertion_command_count"),
                    inventory.assertion_command_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("delete_command_count"),
                    inventory.delete_command_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("declared_collection_count"),
                    inventory.declared_collection_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("relation_kind_count"),
                    inventory.relation_kind_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("declared_aspect_touch_count"),
                    inventory.declared_aspect_touch_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("declared_aspect_operation_count"),
                    inventory.declared_aspect_operation_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("touched_aspect_count"),
                    inventory.touched_aspect_count(),
                )
                .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("row"), row_digests)
                .seal()?;
        Ok(Self {
            kind,
            rows,
            component_count: breadth.component_count(),
            symbolic_entity_declaration_count: breadth.symbolic_entity_declaration_count(),
            symbolic_relation_declaration_count: breadth.symbolic_relation_declaration_count(),
            insert_command_count: inventory.insert_command_count(),
            update_command_count: inventory.update_command_count(),
            assertion_command_count: inventory.assertion_command_count(),
            delete_command_count: inventory.delete_command_count(),
            declared_collection_count: inventory.declared_collection_count(),
            relation_kind_count: inventory.relation_kind_count(),
            declared_aspect_touch_count: inventory.declared_aspect_touch_count(),
            declared_aspect_operation_count: inventory.declared_aspect_operation_count(),
            touched_aspect_count: inventory.touched_aspect_count(),
            descriptor_digest,
        })
    }

    pub fn kind(&self) -> WorthQueryGraphTouchDescriptorKind {
        self.kind
    }

    pub fn rows(&self) -> &[WorthQueryGraphTouchDescriptorRow] {
        &self.rows
    }

    pub fn component_count(&self) -> usize {
        self.component_count
    }

    pub fn symbolic_entity_declaration_count(&self) -> usize {
        self.symbolic_entity_declaration_count
    }

    pub fn symbolic_relation_declaration_count(&self) -> usize {
        self.symbolic_relation_declaration_count
    }

    pub fn insert_command_count(&self) -> usize {
        self.insert_command_count
    }

    pub fn update_command_count(&self) -> usize {
        self.update_command_count
    }

    pub fn assertion_command_count(&self) -> usize {
        self.assertion_command_count
    }

    pub fn delete_command_count(&self) -> usize {
        self.delete_command_count
    }

    pub fn declared_collection_count(&self) -> usize {
        self.declared_collection_count
    }

    pub fn relation_kind_count(&self) -> usize {
        self.relation_kind_count
    }

    pub fn declared_aspect_touch_count(&self) -> usize {
        self.declared_aspect_touch_count
    }

    pub fn declared_aspect_operation_count(&self) -> usize {
        self.declared_aspect_operation_count
    }

    pub fn touched_aspect_count(&self) -> usize {
        self.touched_aspect_count
    }

    pub fn touches_target_collection(
        &self,
        collection: &WorthQueryMutationTargetCollectionIdentity,
    ) -> bool {
        self.rows
            .iter()
            .any(|row| row.touches_declared_collection(collection))
    }

    pub fn touches_relation_kind_id(&self, relation_kind_id: KindId) -> bool {
        self.rows
            .iter()
            .any(|row| row.relation_kind_id() == Some(relation_kind_id))
    }

    pub fn touches_declared_aspect_operation(
        &self,
        operation: &WorthQueryAspectMutationOperation,
    ) -> bool {
        self.rows.iter().any(|row| {
            row.declared_aspect_operations()
                .iter()
                .any(|item| item == operation)
        })
    }

    pub fn touches_aspect(&self, aspect_touch: &WorthQueryAspectTouch) -> bool {
        self.rows.iter().any(|row| {
            row.declared_aspect_operations()
                .iter()
                .any(|operation| operation.aspect_touch() == aspect_touch)
                || row
                    .admitted_touched_aspects()
                    .iter()
                    .any(|touch| touch == aspect_touch)
        })
    }

    pub fn descriptor_digest(&self) -> &str {
        self.descriptor_digest.as_str()
    }

    pub fn descriptor_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.descriptor_digest
    }
}

fn sorted_unique_operations(
    values: impl IntoIterator<Item = WorthQueryAspectMutationOperation>,
) -> Vec<WorthQueryAspectMutationOperation> {
    values
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_unique_touches(
    values: impl IntoIterator<Item = WorthQueryAspectTouch>,
) -> Vec<WorthQueryAspectTouch> {
    values
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
