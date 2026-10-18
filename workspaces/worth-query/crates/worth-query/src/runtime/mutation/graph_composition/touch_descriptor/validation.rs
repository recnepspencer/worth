use std::collections::BTreeSet;

use crate::runtime::{
    WorthQueryGraphCompositionBreadth, WorthQueryGraphCompositionProgram,
    WorthQueryGraphCompositionProgramStepKind, WorthQueryWriteCommand,
};

use super::denial::{
    WorthQueryGraphTouchDescriptorDenial, WorthQueryGraphTouchDescriptorDenialKind,
};

pub(super) fn validate_graph_touch_descriptor_inputs(
    program: &WorthQueryGraphCompositionProgram,
    breadth: &WorthQueryGraphCompositionBreadth,
    commands: &[WorthQueryWriteCommand],
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    if program.is_empty() {
        return validate_empty_program_breadth(breadth);
    }
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::ProgramComponentCountMismatch,
        program.component_count(),
        commands.len(),
        "graph composition program component count must match command count",
    )?;
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::BreadthProgramComponentCountMismatch,
        breadth.component_count(),
        program.component_count(),
        "graph composition breadth component count must match program component count",
    )?;
    validate_program_step_indexes(program, commands)?;
    validate_program_command_semantics(program, commands)?;
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::SymbolicEntityDeclarationCountMismatch,
        breadth.symbolic_entity_declaration_count(),
        symbolic_entity_declaration_count(program),
        "graph composition breadth symbolic entity count must match program declarations",
    )?;
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::SymbolicRelationDeclarationCountMismatch,
        breadth.symbolic_relation_declaration_count(),
        symbolic_relation_declaration_count(program),
        "graph composition breadth symbolic relation count must match program declarations",
    )
}

fn validate_empty_program_breadth(
    breadth: &WorthQueryGraphCompositionBreadth,
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::BreadthProgramComponentCountMismatch,
        breadth.component_count(),
        0,
        "ordinary batch descriptor may not carry graph breadth without a graph program",
    )?;
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::SymbolicEntityDeclarationCountMismatch,
        breadth.symbolic_entity_declaration_count(),
        0,
        "ordinary batch descriptor may not carry symbolic entity breadth",
    )?;
    require_equal(
        WorthQueryGraphTouchDescriptorDenialKind::SymbolicRelationDeclarationCountMismatch,
        breadth.symbolic_relation_declaration_count(),
        0,
        "ordinary batch descriptor may not carry symbolic relation breadth",
    )
}

fn validate_program_step_indexes(
    program: &WorthQueryGraphCompositionProgram,
    commands: &[WorthQueryWriteCommand],
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    let mut indexes = BTreeSet::new();
    for step in program.steps() {
        if step.component_index() >= commands.len() {
            return Err(WorthQueryGraphTouchDescriptorDenial::new(
                WorthQueryGraphTouchDescriptorDenialKind::ProgramStepIndexOutOfBounds,
                format!(
                    "graph composition program step index {} exceeds command count {}",
                    step.component_index(),
                    commands.len()
                ),
            ));
        }
        if !indexes.insert(step.component_index()) {
            return Err(WorthQueryGraphTouchDescriptorDenial::new(
                WorthQueryGraphTouchDescriptorDenialKind::DuplicateProgramStepIndex,
                format!(
                    "graph composition program step index {} appears more than once",
                    step.component_index()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_program_command_semantics(
    program: &WorthQueryGraphCompositionProgram,
    commands: &[WorthQueryWriteCommand],
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    for step in program.steps() {
        let command = &commands[step.component_index()];
        validate_program_command_collection(step, command)?;
        validate_program_command_symbol(step, command)?;
        validate_program_command_mutation_family(step, command)?;
    }
    Ok(())
}

fn validate_program_command_collection(
    step: &crate::runtime::WorthQueryGraphCompositionProgramStep,
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    if let (Some(command_collection), Some(step_collection)) = (
        command.declared_collection_identity(),
        step.declared_collection_identity(),
    ) && command_collection.as_str() != step_collection.as_str()
    {
        return Err(WorthQueryGraphTouchDescriptorDenial::new(
            WorthQueryGraphTouchDescriptorDenialKind::ProgramCommandCollectionMismatch,
            format!(
                "program collection `{}` does not match command collection `{}`",
                step_collection.as_str(),
                command_collection.as_str()
            ),
        ));
    }
    Ok(())
}

fn validate_program_command_symbol(
    step: &crate::runtime::WorthQueryGraphCompositionProgramStep,
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    let Some(declared_symbol) = step.declared_symbol() else {
        return Ok(());
    };
    let Some(reference) = command.symbolic_target_reference() else {
        return Ok(());
    };
    if reference.symbol() == declared_symbol {
        return Ok(());
    }
    Err(WorthQueryGraphTouchDescriptorDenial::new(
        WorthQueryGraphTouchDescriptorDenialKind::ProgramCommandSymbolMismatch,
        format!(
            "program symbol `{declared_symbol}` does not match command symbol `{}`",
            reference.symbol()
        ),
    ))
}

fn validate_program_command_mutation_family(
    step: &crate::runtime::WorthQueryGraphCompositionProgramStep,
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    let expected = expected_mutation_family(step.kind());
    if expected == command.mutation_family() {
        return Ok(());
    }
    Err(WorthQueryGraphTouchDescriptorDenial::new(
        WorthQueryGraphTouchDescriptorDenialKind::ProgramCommandMutationFamilyMismatch,
        format!(
            "program step `{}` expects mutation family `{}`, got `{}`",
            step.kind().as_str(),
            expected.as_str(),
            command.mutation_family().as_str()
        ),
    ))
}

fn expected_mutation_family(
    kind: WorthQueryGraphCompositionProgramStepKind,
) -> crate::runtime::WorthQueryMutationFamily {
    use crate::runtime::WorthQueryMutationFamily;
    match kind {
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
        | WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
        | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration => {
            WorthQueryMutationFamily::Insert
        }
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
        | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetFollowupMutation
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetarget
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetSupersession
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedFollowupMutation
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetarget
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession => {
            WorthQueryMutationFamily::Update
        }
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetirement
        | WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetirement => {
            WorthQueryMutationFamily::Delete
        }
    }
}

fn symbolic_entity_declaration_count(program: &WorthQueryGraphCompositionProgram) -> usize {
    program
        .steps()
        .iter()
        .filter(|step| {
            matches!(
                step.kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
            )
        })
        .count()
}

fn symbolic_relation_declaration_count(program: &WorthQueryGraphCompositionProgram) -> usize {
    program
        .steps()
        .iter()
        .filter(|step| {
            matches!(
                step.kind(),
                WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
                    | WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration
            )
        })
        .count()
}

fn require_equal(
    kind: WorthQueryGraphTouchDescriptorDenialKind,
    actual: usize,
    expected: usize,
    message: &'static str,
) -> Result<(), WorthQueryGraphTouchDescriptorDenial> {
    if actual == expected {
        return Ok(());
    }
    Err(WorthQueryGraphTouchDescriptorDenial::new(
        kind,
        format!("{message}: actual={actual}; expected={expected}"),
    ))
}
