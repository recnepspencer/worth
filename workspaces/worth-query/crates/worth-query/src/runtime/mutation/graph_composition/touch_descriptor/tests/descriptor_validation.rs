use crate::runtime::{
    WorthQueryGraphCompositionBreadth, WorthQueryGraphCompositionProgram,
    WorthQueryGraphCompositionProgramStep, WorthQueryGraphCompositionProgramStepKind,
    WorthQueryGraphTouchDescriptor, WorthQueryGraphTouchDescriptorDenialKind,
    WorthQueryMutationMetadata, WorthQueryMutationTargetCollectionIdentity,
    WorthQuerySymbolicTargetReference, WorthQueryWriteCommand,
};

use super::fixtures::{one_step_delete_program, touch};

#[test]
fn mismatched_program_command_breadth_is_denied_before_descriptor_identity() {
    let (mut commands, breadth, program) = one_step_delete_program(
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
        "topology.edge",
        "edge",
        vec!["weight"],
    );
    commands.push(commands[0].clone());
    let denial = WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(
        &program, &breadth, &commands,
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryGraphTouchDescriptorDenialKind::ProgramComponentCountMismatch
    );
}

#[test]
fn mismatched_symbolic_relation_breadth_is_denied_before_descriptor_identity() {
    let (commands, _breadth, program) = one_step_delete_program(
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
        "topology.edge",
        "edge",
        vec!["weight"],
    );
    let dishonest_breadth = WorthQueryGraphCompositionBreadth::new(1, 0, 1);

    let denial = WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(
        &program,
        &dishonest_breadth,
        &commands,
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryGraphTouchDescriptorDenialKind::SymbolicRelationDeclarationCountMismatch
    );
}

#[test]
fn mismatched_program_command_collection_is_denied_with_matching_counts() {
    let command = WorthQueryWriteCommand::DeleteSymbolicAspects {
        reference: reference("edge", "topology.edge"),
        touched_aspects: vec![touch("weight")],
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
    };
    let breadth = WorthQueryGraphCompositionBreadth::new(1, 0, 0);
    let program = WorthQueryGraphCompositionProgram::new(
        vec![WorthQueryGraphCompositionProgramStep::new(
            0,
            WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
            Some(target_collection("topology.face")),
            Some("edge".to_string()),
        )],
        &breadth,
    )
    .expect("fixture digest must fit");

    let denial = WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(
        &program,
        &breadth,
        &[command],
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryGraphTouchDescriptorDenialKind::ProgramCommandCollectionMismatch
    );
}

#[test]
fn mismatched_program_command_symbol_is_denied_with_matching_counts() {
    let command = WorthQueryWriteCommand::DeleteSymbolicAspects {
        reference: reference("edge", "topology.edge"),
        touched_aspects: vec![touch("weight")],
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
    };
    let breadth = WorthQueryGraphCompositionBreadth::new(1, 0, 0);
    let program = WorthQueryGraphCompositionProgram::new(
        vec![WorthQueryGraphCompositionProgramStep::new(
            0,
            WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
            Some(target_collection("topology.edge")),
            Some("other-edge".to_string()),
        )],
        &breadth,
    )
    .expect("fixture digest must fit");

    let denial = WorthQueryGraphTouchDescriptor::from_authoritative_mutation_batch(
        &program,
        &breadth,
        &[command],
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryGraphTouchDescriptorDenialKind::ProgramCommandSymbolMismatch
    );
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
