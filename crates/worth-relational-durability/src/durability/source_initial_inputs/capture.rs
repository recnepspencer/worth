//! Capture actual original producer DATA before the initialization durable step.
use super::{decoding::layout, frame::*, image::*, recorded::*};
use crate::durability::data::branch_root_schema_images::DurableBranchRootSchemaImage;
use crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::transition::original_inputs::OriginalEmptySchemaObservation;
use worth_relational_contracts::internal_port as contracts;

pub fn capture(
    main: &contracts::history::data::coordinates::BranchId,
    original: &OriginalBranchSchemaInputs,
    installation: Option<(
        &OriginalEmptySchemaObservation,
        &OriginalEmptySchemaObservation,
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    )>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<InitialInputsImage>, RelationalInitialInputsRefusal> {
    capture_with(
        main,
        original,
        installation,
        request,
        worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority::recorded_image,
    )
}

pub(crate) fn capture_complete(
    main: &contracts::history::data::coordinates::BranchId,
    original: &OriginalBranchSchemaInputs,
    installation: Option<(
        &OriginalEmptySchemaObservation,
        &OriginalEmptySchemaObservation,
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    )>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<InitialInputsImage>, RelationalInitialInputsRefusal> {
    capture_with(
        main,
        original,
        installation,
        request,
        worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority::descriptive_image_for_request,
    )
}

pub(crate) fn capture_with(
    main: &contracts::history::data::coordinates::BranchId,
    original: &OriginalBranchSchemaInputs,
    installation: Option<(
        &OriginalEmptySchemaObservation,
        &OriginalEmptySchemaObservation,
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    )>,
    request: &ExecutionRequest<'_, '_>,
    image: fn(
        &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
        &ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    >,
) -> Result<Arc<InitialInputsImage>, RelationalInitialInputsRefusal> {
    use RelationalInitialInputsRefusal as Refusal;
    let birth = original.birth.as_ref().ok_or(Refusal::Missing)?;
    if let Some(accepted) = &original.accepted_initial_inputs {
        if installation.is_none() {
            return Ok(accepted.clone());
        }
        if !matches!(accepted.entries.as_slice(), [InitialInputEntry::Birth(_)]) {
            return Err(Refusal::Malformed);
        }
    }
    let memory = request
        .reserve_retained_memory(layout(2, 3, main.0.len()).ok_or(Refusal::Malformed)?)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut entries = Vec::with_capacity(2);
    let mut schemas: Vec<DurableBranchRootSchemaImage> = Vec::with_capacity(3);
    entries.push(InitialInputEntry::Birth(
        observation(birth, image, request).map_err(Refusal::Native)?,
    ));
    schemas.push(
        image(&birth.schema, request)
            .map_err(Refusal::Native)?
            .into(),
    );
    let installation = installation.or_else(|| {
        original
            .installation
            .as_ref()
            .map(|install| (&install.before, &install.after, install.follows_transition))
    });
    if let Some((before, after, follows_transition)) = installation {
        entries.push(InitialInputEntry::Installation {
            before: observation(before, image, request).map_err(Refusal::Native)?,
            after: observation(after, image, request).map_err(Refusal::Native)?,
            follows_transition,
        });
        schemas.push(
            image(&before.schema, request)
                .map_err(Refusal::Native)?
                .into(),
        );
        schemas.push(
            image(&after.schema, request)
                .map_err(Refusal::Native)?
                .into(),
        );
    }
    schemas.sort_unstable_by_key(|image| image.carrier_digest());
    schemas.dedup_by_key(|image| image.carrier_digest());
    let predecessor = original
        .accepted_initial_inputs
        .as_ref()
        .and_then(|image| image.artifact.as_ref())
        .map(|artifact| artifact.payload_digest());
    let revision = if predecessor.is_some() { 2 } else { 1 };
    let artifact = RelationalSourceInitialInputsArtifact::emit(
        revision,
        &InitialInputsImageRef {
            main,
            revision,
            predecessor,
            entries: &entries,
            schemas: &schemas,
        },
        request,
    )?;
    let image = InitialInputsImage {
        main: main.clone(),
        revision,
        predecessor,
        entries,
        schemas,
        artifact: Some(artifact),
        _memory: memory,
    };
    image.validate()?;
    Ok(Arc::new(image))
}

fn observation(
    original: &OriginalEmptySchemaObservation,
    image: fn(
        &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
        &ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    >,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    RecordedEmptySchemaObservation,
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    Ok(RecordedEmptySchemaObservation {
        generation: original.generation,
        truth_version: original.truth_version,
        lifecycle: original.lifecycle,
        schema_carrier: image(&original.schema, request)?.carrier_digest(),
    })
}
