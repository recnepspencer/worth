use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAuthoredAspectMutation,
    WorthQueryGraphCompositionBreadth, WorthQueryGraphCompositionProgram,
    WorthQueryGraphCompositionProgramStep, WorthQueryGraphCompositionProgramStepKind,
    WorthQueryGraphTouchDescriptor, WorthQueryMutationMetadata,
    WorthQueryMutationTargetCollectionIdentity, WorthQuerySymbolicTargetReference,
    WorthQueryWriteCommand,
};
use worth_foundational::facade::{AspectKey, AspectValue, CanonicalFieldPath, FieldKey};
use worth_relational::facade::identity::KindId;

pub(super) fn descriptor_for_step_kind(
    kind: WorthQueryGraphCompositionProgramStepKind,
) -> WorthQueryGraphTouchDescriptor {
    let (commands, breadth, program) = if matches!(
        kind,
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation
            | WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetFollowupMutation
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetarget
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetSupersession
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedFollowupMutation
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetarget
            | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession
    ) {
        one_step_update_program(kind, "topology.edge", "edge", "weight")
    } else {
        one_step_delete_program(kind, "topology.edge", "edge", vec!["weight"])
    };
    WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(&program, &breadth, &commands)
        .unwrap()
}

pub(super) fn descriptor_for_collection(collection: &str) -> WorthQueryGraphTouchDescriptor {
    let (commands, breadth, program) = one_step_delete_program(
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
        collection,
        "edge",
        vec!["weight"],
    );
    WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(&program, &breadth, &commands)
        .unwrap()
}

pub(super) fn descriptor_for_relation_kind_id(
    relation_kind_id: KindId,
) -> WorthQueryGraphTouchDescriptor {
    let (commands, breadth, program) = one_step_delete_program_with_relation_kind_id(
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
        "topology.edge",
        "edge",
        vec!["weight"],
        relation_kind_id,
    );
    WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(&program, &breadth, &commands)
        .unwrap()
}

pub(super) fn descriptor_for_touched_paths(
    touched_paths: Vec<&str>,
) -> WorthQueryGraphTouchDescriptor {
    let (commands, breadth, program) = one_step_delete_program(
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
        "topology.edge",
        "edge",
        touched_paths,
    );
    WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(&program, &breadth, &commands)
        .unwrap()
}

pub(super) fn one_step_delete_program(
    kind: WorthQueryGraphCompositionProgramStepKind,
    collection: &str,
    symbol: &str,
    touched_paths: Vec<&str>,
) -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    let command = WorthQueryWriteCommand::DeleteSymbolicAspects {
        reference: reference(symbol, collection),
        touched_aspects: touched_paths.into_iter().map(touch).collect(),
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
    };
    one_step_program(kind, collection, symbol, command)
}

pub(super) fn one_step_delete_program_with_relation_kind_id(
    kind: WorthQueryGraphCompositionProgramStepKind,
    collection: &str,
    symbol: &str,
    touched_paths: Vec<&str>,
    relation_kind_id: KindId,
) -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    let command = WorthQueryWriteCommand::DeleteSymbolicAspects {
        reference: reference(symbol, collection),
        touched_aspects: touched_paths.into_iter().map(touch).collect(),
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
    };
    one_step_program_with_relation_kind_id(
        kind,
        collection,
        symbol,
        command,
        Some(relation_kind_id),
    )
}

pub(super) fn one_step_update_program(
    kind: WorthQueryGraphCompositionProgramStepKind,
    collection: &str,
    symbol: &str,
    touch_fixture: &str,
) -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    let command = WorthQueryWriteCommand::UpdateSymbolicAspects {
        reference: reference(symbol, collection),
        aspects: vec![
            WorthQueryAuthoredAspectMutation::new(touch(touch_fixture), int_value(1)).unwrap(),
        ],
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
        continuity_intent: None,
    };
    one_step_program(kind, collection, symbol, command)
}

pub(super) fn touch(touch_fixture: &str) -> crate::runtime::WorthQueryAspectTouch {
    native_touch(touch_fixture)
}

pub(super) fn set_operation(touch_fixture: &str) -> WorthQueryAspectMutationOperation {
    WorthQueryAspectMutationOperation::set(touch(touch_fixture))
}

fn int_value(value: i64) -> AspectValue {
    AspectValue::Int64(value)
}

fn native_touch(aspect_fixture: &str) -> crate::runtime::WorthQueryAspectTouch {
    let mut segments = aspect_fixture.split('.');
    let aspect_key = AspectKey::new(
        segments
            .next()
            .expect("test touch fixture should name an aspect"),
    )
    .expect("test aspect key should admit");
    let field_segments = segments
        .map(|segment| FieldKey::new(segment).expect("test field key should admit"))
        .collect::<Vec<_>>();
    if field_segments.is_empty() {
        crate::runtime::WorthQueryAspectTouch::whole_aspect(aspect_key)
    } else {
        crate::runtime::WorthQueryAspectTouch::aspect_field_path(
            aspect_key,
            CanonicalFieldPath::new(field_segments).expect("test field path should admit"),
        )
    }
}

fn one_step_program(
    kind: WorthQueryGraphCompositionProgramStepKind,
    collection: &str,
    symbol: &str,
    command: WorthQueryWriteCommand,
) -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    one_step_program_with_relation_kind_id(kind, collection, symbol, command, None)
}

fn one_step_program_with_relation_kind_id(
    kind: WorthQueryGraphCompositionProgramStepKind,
    collection: &str,
    symbol: &str,
    command: WorthQueryWriteCommand,
    relation_kind_id: Option<KindId>,
) -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    let breadth = WorthQueryGraphCompositionBreadth::new(
        1,
        symbolic_entity_declaration_count(kind),
        symbolic_relation_declaration_count(kind),
    );
    let mut step = WorthQueryGraphCompositionProgramStep::new(
        0,
        kind,
        Some(target_collection(collection)),
        Some(symbol.to_string()),
    );
    if let Some(relation_kind_id) = relation_kind_id {
        step = step.with_relation_kind_id(relation_kind_id);
    }
    let program = WorthQueryGraphCompositionProgram::new(vec![step], &breadth)
        .expect("fixture digest must fit");
    (vec![command], breadth, program)
}

fn reference(symbol: &str, collection: &str) -> WorthQuerySymbolicTargetReference {
    WorthQuerySymbolicTargetReference::new(symbol)
        .unwrap()
        .in_target_collection(collection)
        .unwrap()
}

fn target_collection(collection: &str) -> WorthQueryMutationTargetCollectionIdentity {
    WorthQueryMutationTargetCollectionIdentity::new("graph-composition-test", collection)
        .expect("fixture digest must fit")
}

fn symbolic_entity_declaration_count(kind: WorthQueryGraphCompositionProgramStepKind) -> usize {
    if matches!(
        kind,
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
    ) {
        1
    } else {
        0
    }
}

fn symbolic_relation_declaration_count(kind: WorthQueryGraphCompositionProgramStepKind) -> usize {
    if matches!(
        kind,
        WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
            | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration
    ) {
        1
    } else {
        0
    }
}
