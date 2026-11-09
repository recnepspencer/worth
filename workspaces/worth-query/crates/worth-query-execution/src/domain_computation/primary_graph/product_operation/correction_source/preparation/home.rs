//! Persist the admitted command snapshot without carrying its live selection.
use super::custody::PreparedCorrectionCustody;
use super::*;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation;
use crate::domain_computation::primary_graph::retained_correction::record::{
    PreparedCorrectionSnapshot, RecordedCorrectionTarget, RecordedLocation, RecordedScope,
};

pub(super) fn prepare<S, B>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    source: &WorthQuerySelectedCorrectionSource<'_, S>,
    input: &ApplicationEncodedInput<B::InputBinding>,
    intent: &WorthQueryCorrectionIntent,
    key: &WorthQueryCorrectionCommandKey,
    actor: EntityId,
    scope: &crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector,
    program: &WorthQueryProgramSupportInterpretation,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PreparedCorrectionCustody, WorthQueryCorrectionDerivationDenial>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: serde::Serialize,
{
    use WorthQueryCorrectionDerivationDenial as D;
    let material = source
        .occurrence()
        .home_material()
        .ok_or(D::UnsupportedSourceMeaning)?;
    let branch = target.product().observation();
    let owner = branch.owner_identity();
    let durable = owner.durable().ok_or(D::ForeignSelection)?;
    let retained = input
        .clone()
        .retain_canonical_input()
        .map_err(D::OriginalInputDecode)?;
    let parent = branch
        .basis()
        .relational_basis()
        .observation()
        .commit_receipt()
        .cloned()
        .ok_or(D::MissingTargetParent)?;
    let snapshot = PreparedCorrectionSnapshot {
        key: key.identity(),
        intent: intent.identity(),
        action: B::IDENTITY.to_owned(),
        input: retained.recorded_bytes().to_vec(),
        input_identity: *input.identity(),
        actor,
        scope: RecordedScope {
            subject: scope.subject(),
            value: scope.value().clone(),
            location: RecordedLocation {
                aspect: scope.locator().aspect().aspect_key().as_str().to_owned(),
                fields: scope
                    .locator()
                    .field_path()
                    .fields()
                    .iter()
                    .map(|field| field.as_str().to_owned())
                    .collect(),
            },
        },
        target: RecordedCorrectionTarget {
            owner: durable,
            branch: branch.branch_identity().name().as_str().to_owned(),
            incarnation: branch.lifecycle_incarnation().ordinal(),
            generation: branch.reference_generation().get(),
            commit: branch.selected_commit().ordinal(),
            relational: parent.clone(),
            program: *program.revision().as_bytes(),
        },
    };
    let pin = target
        .application()
        .runtime
        .primary_provider
        .prepare_correction_command(material.capture.original, snapshot, request)
        .map_err(D::Material)?;
    Ok(PreparedCorrectionCustody::Home {
        pin,
        fence: owner.home_incarnation_fence().ok_or(D::ForeignSelection)?,
        world: source.ancestry().source().identity().clone(),
        commit: source.occurrence().commit_reference().clone(),
        _material: material.clone(),
    })
}
