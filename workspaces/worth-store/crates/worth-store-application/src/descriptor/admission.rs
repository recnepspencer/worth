//! Select the genuine compiled predecessor before any descriptor effect.
use super::{
    binding, codec, transition_required, ApplicationHomeDescriptorDenial as Denial, HomeDescriptor,
    PreparedDescriptor, MAXIMUM_DESCRIPTOR_BYTES,
};
use crate::open::{
    readmission::transition::{HomeTransitionAdmission, ReadmittedHomeProgramTransitions},
    ApplicationHomeOpenRefusal as Refusal,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    ApplicationBuildIdentity, ApplicationHomeBinding, HomeIdentity, HomeIncarnation,
};
use worth_relational::facade::runtime::RelationalRuntimeDefinition;
use worth_store::physical_runtime::{
    AnchoredRecordName, RecoveryStorePresence, ServingPhysicalRuntime,
};

pub(crate) fn prepare_admission(
    serving: &ServingPhysicalRuntime,
    origin: RecoveryStorePresence,
    definition: RelationalRuntimeDefinition,
    binding: ApplicationHomeBinding,
    predecessor: Option<crate::home_request::HomeTransitionPredecessorInput>,
    build: ApplicationBuildIdentity,
    source: &crate::checkpoint::SelectedHomeSource,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedDescriptor, Refusal> {
    crate::request_stage::prepare(
        execution,
        || {
            prepare(
                serving,
                origin,
                definition,
                binding,
                predecessor,
                build,
                source,
                execution,
            )
        },
        |cause| Refusal::Descriptor(Denial::Execution(cause)),
    )
}

fn prepare(
    serving: &ServingPhysicalRuntime,
    origin: RecoveryStorePresence,
    definition: RelationalRuntimeDefinition,
    binding: ApplicationHomeBinding,
    predecessor: Option<crate::home_request::HomeTransitionPredecessorInput>,
    build: ApplicationBuildIdentity,
    source: &crate::checkpoint::SelectedHomeSource,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedDescriptor, Refusal> {
    execution.consult().map_err(Denial::Execution)?;
    let identity = HomeIdentity::from_observed_bytes(serving.store_identity().bytes())
        .map_err(|_| Denial::IdentityMismatch)?;
    let config = definition.configuration();
    let runtime = &config.execution.runtime_name;
    let profile = format!("{:?}", config.profile);
    let binding_bytes = encode_binding(&binding, execution)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor = AnchoredRecordName::new(crate::registry::HOME_ANCHOR).expect("static home anchor");
    if let Some(legacy) = records
        .read(
            AnchoredRecordName::new("relational.home").unwrap(),
            MAXIMUM_DESCRIPTOR_BYTES as u64,
        )
        .map_err(Denial::Read)?
    {
        let observed = legacy
            .bytes()
            .get(8..10)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes)
            .unwrap_or(0);
        return Err(Denial::UnsupportedVersion { observed }.into());
    }
    let previous = records
        .read(anchor, MAXIMUM_DESCRIPTOR_BYTES as u64)
        .map_err(Denial::Read)?;
    let (generation, incarnation, binding, definition, binding_bytes, transition) = match previous
        .as_ref()
    {
        Some(record) => {
            let (incarnation, recorded, exact) = match codec::admit_record(
                record.bytes(),
                identity.bytes(),
                runtime,
                &profile,
                &binding_bytes.bytes,
            )? {
                codec::DescriptorBindingAdmission::Exact(incarnation) => {
                    (incarnation, &*binding_bytes.bytes, true)
                }
                codec::DescriptorBindingAdmission::TransitionRequired {
                    recorded,
                    incarnation,
                } => (incarnation, recorded, false),
            };
            let history =
                ReadmittedHomeProgramTransitions::read(serving, source, identity, execution)
                    .map_err(Refusal::Transition)?;
            history
                .require_recorded_successor(recorded, &binding)
                .map_err(Refusal::Transition)?;
            let (binding, definition, binding_bytes, transition) = if exact {
                history
                    .require_successor(&binding, &binding)
                    .map_err(Refusal::Transition)?;
                (
                    binding,
                    definition,
                    binding_bytes,
                    HomeTransitionAdmission::Current { history },
                )
            } else {
                let Some(predecessor) = predecessor else {
                    return Err(transition_required::refuse(recorded, binding, execution).into());
                };
                let admitted = predecessor
                    .binding
                    .consume(|admitted| admitted.into_description());
                let predecessor_bytes = encode_binding(&admitted, execution)?;
                if recorded != &*predecessor_bytes.bytes {
                    let rostered =
                        binding::readmission::contains_program(recorded, admitted.selected())?;
                    return Err(if rostered {
                        Denial::TransitionPredecessorMismatch
                    } else {
                        Denial::TransitionPredecessorNotRostered
                    }
                    .into());
                }
                let old_config = predecessor.relational.configuration();
                if old_config.execution.runtime_name != *runtime
                    || format!("{:?}", old_config.profile) != profile
                {
                    return Err(Denial::BindingMismatch.into());
                }
                history
                    .require_successor(&admitted, &binding)
                    .map_err(Refusal::Transition)?;
                (
                    admitted,
                    predecessor.relational,
                    predecessor_bytes,
                    HomeTransitionAdmission::Requested {
                        history,
                        successor: binding,
                    },
                )
            };
            (
                record.version().generation(),
                incarnation
                    .successor()
                    .map_err(|_| Denial::IncarnationExhausted)?,
                binding,
                definition,
                binding_bytes,
                transition,
            )
        }
        None => {
            let incarnation = match origin {
                RecoveryStorePresence::ProvenAbsent => HomeIncarnation::initial(),
                RecoveryStorePresence::ExistingArtifacts => {
                    return Err(Denial::IncompleteInitialization.into())
                }
            };
            let history =
                ReadmittedHomeProgramTransitions::read(serving, source, identity, execution)
                    .map_err(Refusal::Transition)?;
            history
                .require_successor(&binding, &binding)
                .map_err(Refusal::Transition)?;
            (
                0,
                incarnation,
                binding,
                definition,
                binding_bytes,
                HomeTransitionAdmission::Current { history },
            )
        }
    };
    let descriptor = HomeDescriptor {
        identity,
        incarnation,
        binding,
        build,
    };
    let config = definition.configuration();
    let runtime = &config.execution.runtime_name;
    let profile = format!("{:?}", config.profile);
    let size = codec::size(runtime, &profile, binding_bytes.bytes.len())
        .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
        .ok_or(Denial::SizeLimit)?;
    let payload = crate::frame_encoding::encode(size, execution, |bytes| {
        codec::emit(&descriptor, runtime, &profile, &binding_bytes.bytes, bytes)
    })
    .map_err(Denial::Execution)?;
    Ok(PreparedDescriptor {
        descriptor,
        payload,
        generation,
        definition,
        transition,
    })
}

fn encode_binding(
    binding: &ApplicationHomeBinding,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<crate::frame_encoding::EncodedHomeFrame, Denial> {
    let size = binding::size(binding)
        .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
        .ok_or(Denial::SizeLimit)?;
    crate::frame_encoding::encode(size, execution, |bytes| binding::emit(binding, bytes))
        .map_err(Denial::Execution)
}
