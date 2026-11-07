use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;

use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryGraphCompositionProgram,
    WorthQueryGraphCompositionProgramStep, WorthQueryWriteCommand,
};

use super::super::lifecycle_family::WorthQueryGraphTouchLifecycleFamily;
use super::{WorthQueryGraphTouchDescriptorRow, WorthQueryGraphTouchDescriptorRowInput};

pub(in super::super) fn derive_command_touch_rows(
    program: &WorthQueryGraphCompositionProgram,
    commands: &[WorthQueryWriteCommand],
) -> Result<Vec<WorthQueryGraphTouchDescriptorRow>, IdentityDerivationDenial> {
    let steps_by_component = program
        .steps()
        .iter()
        .map(|step| (step.component_index(), step))
        .collect::<BTreeMap<_, _>>();
    Ok(commands
        .iter()
        .enumerate()
        .map(|(component_index, command)| {
            let step = steps_by_component.get(&component_index).copied();
            row_from_command(component_index, command, step)
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn row_from_command(
    component_index: usize,
    command: &WorthQueryWriteCommand,
    step: Option<&WorthQueryGraphCompositionProgramStep>,
) -> Result<WorthQueryGraphTouchDescriptorRow, IdentityDerivationDenial> {
    let program_step_kind = step.map(WorthQueryGraphCompositionProgramStep::kind);
    let lifecycle_family =
        program_step_kind.map(WorthQueryGraphTouchLifecycleFamily::from_program_step_kind);
    let declared_collection = command
        .declared_collection_identity()
        .or_else(|| step.and_then(|step| step.declared_collection_identity().cloned()));
    let relation_kind_id = step.and_then(WorthQueryGraphCompositionProgramStep::relation_kind_id);
    let declared_symbol = step.and_then(|step| step.declared_symbol().map(str::to_string));
    Ok(WorthQueryGraphTouchDescriptorRow::new(
        WorthQueryGraphTouchDescriptorRowInput {
            component_index,
            mutation_family: command.mutation_family(),
            read_verb: None,
            program_step_kind,
            lifecycle_family,
            declared_collection,
            relation_kind_id,
            declared_symbol,
            declared_aspect_operations: declared_aspect_operations(command),
            touched_aspects: sorted_unique_touches(
                command.admitted_touched_aspects().iter().cloned(),
            ),
            has_symbolic_target_reference: command.symbolic_target_reference().is_some(),
            has_existing_truth_binding: command.existing_truth_binding().is_some(),
            symbolic_aspect_reference_count: command.symbolic_aspect_references().len(),
        },
    )?)
}

fn declared_aspect_operations(
    command: &WorthQueryWriteCommand,
) -> Vec<WorthQueryAspectMutationOperation> {
    sorted_unique_operations(command.declared_aspect_operations())
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
    values: impl IntoIterator<Item = crate::runtime::WorthQueryAspectTouch>,
) -> Vec<crate::runtime::WorthQueryAspectTouch> {
    values
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
