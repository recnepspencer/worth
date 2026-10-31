mod binding;
mod codec;

use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    ApplicationHomeBinding, DurableArtifactRetention, HomeIdentity, HomeIncarnation, RetentionClass,
};
use worth_relational::facade::runtime::RelationalRuntimeDefinition;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordReadFailure, AnchoredRecordsDenial, PhysicalMutationDeadline,
    RecoveryStorePresence, ServingPhysicalRuntime,
};

const MAXIMUM_DESCRIPTOR_BYTES: usize = 64 << 10;

pub(crate) struct HomeDescriptor {
    pub(crate) identity: HomeIdentity,
    pub(crate) incarnation: HomeIncarnation,
    pub(crate) binding: ApplicationHomeBinding,
    pub(crate) build: worth_foundational::facade::ApplicationBuildIdentity,
}

impl DurableArtifactRetention for HomeDescriptor {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Authoritative;
}

#[derive(Debug)]
pub enum ApplicationHomeDescriptorDenial {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    Publication(AnchoredRecordPublicationFailure),
    Execution(WorkCeilingDenial),
    Deadline(crate::ApplicationHomeDeadlineDenial),
    Malformed,
    SizeLimit,
    UnsupportedVersion { observed: u16 },
    UnsupportedBinding { tag: u16 },
    IdentityMismatch,
    ApplicationBindingMissing,
    BuildBindingMissing,
    BindingMismatch,
    IncompleteInitialization,
    IncarnationExhausted,
}

struct PreparedDescriptor {
    descriptor: HomeDescriptor,
    payload: crate::frame_encoding::EncodedHomeFrame,
    generation: u64,
}

pub(crate) fn admit(
    serving: &ServingPhysicalRuntime,
    origin: RecoveryStorePresence,
    definition: &RelationalRuntimeDefinition,
    binding: ApplicationHomeBinding,
    build: worth_foundational::facade::ApplicationBuildIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<HomeDescriptor, ApplicationHomeDescriptorDenial> {
    use ApplicationHomeDescriptorDenial as Denial;
    let prepared = crate::request_stage::prepare(
        execution,
        || prepare(serving, origin, definition, binding, build, execution),
        Denial::Execution,
    )?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor = AnchoredRecordName::new(crate::registry::HOME_ANCHOR).expect("static home anchor");
    let PreparedDescriptor {
        descriptor,
        payload,
        generation,
    } = prepared;
    let milliseconds =
        crate::deadline::milliseconds(execution, 30_000).map_err(Denial::Deadline)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(milliseconds)
        .expect("admitted positive descriptor duration");
    execution.consult().map_err(Denial::Execution)?;
    let crate::frame_encoding::EncodedHomeFrame {
        bytes,
        memory: _memory,
    } = payload;
    records
        .publish(AnchoredRecordPublication::new(
            anchor,
            generation,
            bytes,
            crate::open::physical_admission::placement(),
            deadline,
        ))
        .map_err(Denial::Publication)?;
    Ok(descriptor)
}

fn prepare(
    serving: &ServingPhysicalRuntime,
    origin: RecoveryStorePresence,
    definition: &RelationalRuntimeDefinition,
    binding: ApplicationHomeBinding,
    build: worth_foundational::facade::ApplicationBuildIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedDescriptor, ApplicationHomeDescriptorDenial> {
    use ApplicationHomeDescriptorDenial as Denial;
    execution.consult().map_err(Denial::Execution)?;
    let identity = HomeIdentity::from_observed_bytes(serving.store_identity().bytes())
        .map_err(|_| Denial::IdentityMismatch)?;
    let config = definition.configuration();
    let runtime = &config.execution.runtime_name;
    let profile = format!("{:?}", config.profile);
    let size = binding::size(&binding)
        .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
        .ok_or(Denial::SizeLimit)?;
    let binding_bytes =
        crate::frame_encoding::encode(size, execution, |bytes| binding::emit(&binding, bytes))
            .map_err(Denial::Execution)?;
    let size = codec::size(runtime, &profile, size)
        .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
        .ok_or(Denial::SizeLimit)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let anchor = AnchoredRecordName::new(crate::registry::HOME_ANCHOR).expect("static home anchor");
    // Retired homes are refused, never adopted or treated as a second source.
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
        return Err(Denial::UnsupportedVersion { observed });
    }
    let previous = records
        .read(anchor, MAXIMUM_DESCRIPTOR_BYTES as u64)
        .map_err(Denial::Read)?;
    let (generation, incarnation) = match previous.as_ref() {
        Some(record) => {
            let incarnation = codec::admit_record(
                record.bytes(),
                identity.bytes(),
                runtime,
                &profile,
                &binding_bytes.bytes,
            )?;
            (
                record.version().generation(),
                incarnation
                    .successor()
                    .map_err(|_| Denial::IncarnationExhausted)?,
            )
        }
        None => match origin {
            RecoveryStorePresence::ProvenAbsent => (0, HomeIncarnation::initial()),
            RecoveryStorePresence::ExistingArtifacts => {
                return Err(Denial::IncompleteInitialization)
            }
        },
    };
    let descriptor = HomeDescriptor {
        identity,
        incarnation,
        binding,
        build,
    };
    let payload = crate::frame_encoding::encode(size, execution, |bytes| {
        codec::emit(&descriptor, runtime, &profile, &binding_bytes.bytes, bytes);
    })
    .map_err(Denial::Execution)?;
    Ok(PreparedDescriptor {
        descriptor,
        payload,
        generation,
    })
}
