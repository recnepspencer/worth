//! Prepare the exact successor descriptor before the guarded final owner effect.
use super::*;
use crate::home_context::HomeContext;

pub(crate) struct PreparedHomeTransitionBinding {
    descriptor: HomeDescriptor,
    frame: crate::frame_encoding::EncodedHomeFrame,
    generation: u64,
}

pub struct ApplicationHomeTransitionBindingRefused {
    cause: ApplicationHomeDescriptorDenial,
    _prepared: PreparedHomeTransitionBinding,
}
impl ApplicationHomeTransitionBindingRefused {
    pub fn reason(&self) -> &ApplicationHomeDescriptorDenial {
        &self.cause
    }
}
impl std::fmt::Debug for ApplicationHomeTransitionBindingRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ApplicationHomeTransitionBindingRefused")
            .field(&self.cause)
            .finish()
    }
}

impl PreparedHomeTransitionBinding {
    pub(crate) fn prepare(
        context: &HomeContext,
        successor: &ApplicationHomeBinding,
        configuration: &worth_relational::facade::runtime::RelationalRuntimeConfig,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ApplicationHomeDescriptorDenial> {
        use ApplicationHomeDescriptorDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let runtime = configuration.execution.runtime_name.as_str();
        use worth_relational::facade::config::RelationalRuntimeProfile as Profile;
        let profile = match configuration.profile {
            Profile::CertificationCore => "CertificationCore",
            Profile::GeometryKernel => "GeometryKernel",
            Profile::ChipSimulation => "ChipSimulation",
            Profile::AiWorkflow => "AiWorkflow",
        };
        let records = context
            .serving
            .anchored_records()
            .map_err(Denial::Unavailable)?;
        let record = records
            .read(
                AnchoredRecordName::new(crate::registry::HOME_ANCHOR).unwrap(),
                MAXIMUM_DESCRIPTOR_BYTES as u64,
            )
            .map_err(Denial::Read)?
            .ok_or(Denial::IncompleteInitialization)?;
        let original = codec::read_record(
            record.bytes(),
            context.descriptor.identity.bytes(),
            runtime,
            profile,
        )?;
        if original.incarnation != context.descriptor.incarnation
            || !binding::describes(original.binding, &context.descriptor.binding)?
        {
            return Err(Denial::BindingMismatch);
        }
        let binding_size = binding::size(successor)
            .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
            .ok_or(Denial::SizeLimit)?;
        let encoded_binding = crate::frame_encoding::encode(binding_size, request, |bytes| {
            binding::emit(successor, bytes)
        })
        .map_err(Denial::Execution)?;
        let metadata = request
            .reserve_retained_memory(binding::allocation_bytes(&encoded_binding.bytes)?)
            .map_err(Denial::Execution)?;
        let descriptor = HomeDescriptor {
            identity: context.descriptor.identity,
            incarnation: context.descriptor.incarnation,
            binding: successor.clone(),
            build: context.descriptor.build,
            _binding_memory: metadata,
        };
        let size = codec::size(runtime, profile, encoded_binding.bytes.len())
            .filter(|size| *size <= MAXIMUM_DESCRIPTOR_BYTES)
            .ok_or(Denial::SizeLimit)?;
        let frame = crate::frame_encoding::encode(size, request, |bytes| {
            codec::emit(&descriptor, runtime, profile, &encoded_binding.bytes, bytes)
        })
        .map_err(Denial::Execution)?;
        Ok(Self {
            descriptor,
            frame,
            generation: record.version().generation(),
        })
    }

    pub(crate) fn publish(
        mut self,
        serving: &ServingPhysicalRuntime,
        descriptor: &mut HomeDescriptor,
        transition: &crate::open::readmission::transition::HomeTransitionOwner,
        plan: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_proof::HomeProgramTransitionBindingCommitted,
        ApplicationHomeTransitionBindingRefused,
    > {
        let published = (|| {
            use ApplicationHomeDescriptorDenial as Denial;
            request.consult().map_err(Denial::Execution)?;
            let records = serving.anchored_records().map_err(Denial::Unavailable)?;
            let milliseconds =
                crate::deadline::milliseconds(request, 30_000).map_err(Denial::Deadline)?;
            records
                .publish(AnchoredRecordPublication::new(
                    AnchoredRecordName::new(crate::registry::HOME_ANCHOR).unwrap(),
                    self.generation,
                    std::mem::take(&mut self.frame.bytes),
                    crate::open::physical_admission::placement(),
                    PhysicalMutationDeadline::after_milliseconds(milliseconds)
                        .expect("positive admitted descriptor deadline"),
                ))
                .map_err(Denial::Publication)?;
            Ok(())
        })();
        match published {
            Ok(()) => {
                *descriptor = self.descriptor;
                Ok(transition.acknowledge_binding(&descriptor.binding, plan))
            }
            Err(cause) => Err(ApplicationHomeTransitionBindingRefused {
                cause,
                _prepared: self,
            }),
        }
    }
}
