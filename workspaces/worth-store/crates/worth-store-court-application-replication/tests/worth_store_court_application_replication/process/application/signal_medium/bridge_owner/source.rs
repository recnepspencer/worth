//! Seed original owner artifacts from a separate memory-only program capture.
use super::{media, program};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
pub(super) struct Reference {
    pub(super) program: Box<program::Program>,
    pub(super) definitions: Vec<SignalDefinitionTransitionArtifact>,
    pub(super) advances: Vec<SignalBranchAdvanceArtifact>,
}
pub(super) fn reference(execution: &ExecutionRequest<'_, '_>) -> Box<Reference> {
    reference_from_program(program::program(), execution)
}
pub(super) fn reference_from_program(
    program: Box<program::Program>,
    execution: &ExecutionRequest<'_, '_>,
) -> Box<Reference> {
    let mut definitions = Vec::new();
    let mut advances = Vec::new();
    let mut parent = None;
    for ordinal in 1..=3 {
        let branch = DurableSignalBranchIdentity::from_recorded(media::owner(), ordinal);
        let occurrences: Vec<_> = program
            .lowerings
            .iter()
            .enumerate()
            .map(|(index, lowering)| {
                let node = lowering.signal_node();
                DurableSignalConditionalDefinitionOccurrence::from_recorded(
                    DurableSignalNodeIdentity::from_recorded(
                        branch,
                        node.index(),
                        node.generation(),
                    ),
                    DurableSignalConditionalOccurrence::from_recorded(
                        branch.owner(),
                        ordinal * 2 + index as u64 + 1,
                    ),
                )
                .unwrap()
            })
            .collect();
        let definition = program
            .builder
            .capture_authored_definition(
                execution,
                SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), ordinal),
                branch,
                &occurrences,
            )
            .unwrap();
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(
                DurableSignalCellIncarnation::from_recorded(branch, ordinal),
                ordinal,
            ),
            None,
            DurableSignalAdmissionIdentity::from_recorded(branch.owner(), ordinal),
            definition.reference(),
        )
        .unwrap();
        advances.push(
            SignalBranchAdvanceArtifact::capture(
                execution,
                SignalBranchAdvanceIdentity::from_recorded(branch.owner(), ordinal),
                descriptor.clone(),
                None,
                SignalAuthoredBranchChange::Created {
                    name: format!("bridge-branch-{ordinal}"),
                    parent: parent.clone(),
                },
            )
            .unwrap(),
        );
        definitions.push(definition);
        parent = Some(descriptor);
    }
    Box::new(Reference {
        program,
        definitions,
        advances,
    })
}
pub(super) fn seed(
    medium: &dyn SignalDurableMedium,
    reference: &Reference,
    execution: &ExecutionRequest<'_, '_>,
) {
    for (definition, advance) in reference.definitions.iter().zip(&reference.advances) {
        let committed = super::super::drive(|| medium.advance_definition(definition, execution));
        assert!(matches!(
            committed.fate(),
            DurablePublicationFate::Committed
        ));
        let committed = super::super::drive(|| medium.advance_branch(advance, execution));
        assert!(matches!(
            committed.fate(),
            DurablePublicationFate::Committed
        ));
    }
}
