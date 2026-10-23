//! Canonical preparation receipts and admission by the original request.
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationComputationPartitionIdentityDenial, CanonicalEncodingCharge,
    },
    application_schema::ApplicationValueEncodeDenial,
};

use super::super::super::request_local::{assert_request_local, RequestLocal};
use super::super::{
    execution_denial::checkpoint_denial, request_execution::QueryRequestExecution,
    WorthQueryAdvancementDenial, WorthQueryManagedComputationCheckpointDenial as CheckpointDenial,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use super::WorthQueryPartitionedComputationDenial as Denial;

/// Records canonical preparation work for native full-equivalent reuse.
/// This is a receipt counter, never a separate admission ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PreparationWork {
    spent: Option<u64>,
    request_local: RequestLocal,
}

assert_request_local!(PreparationWork, EncodingMeter<'static>);

impl PreparationWork {
    pub(super) const fn new(accounted: bool) -> Self {
        Self {
            spent: if accounted { Some(0) } else { None },
            request_local: PhantomData,
        }
    }

    pub(super) fn spend(&mut self, work: Option<u64>) -> Result<(), Resource> {
        let Some(spent) = self.spent else {
            return Ok(());
        };
        self.spent = Some(
            spent
                .checked_add(work.ok_or(Resource::WorkCounterOverflow)?)
                .ok_or(Resource::WorkCounterOverflow)?,
        );
        Ok(())
    }

    pub(super) const fn spent(self) -> Option<u64> {
        self.spent
    }

    pub(super) fn spent_since(self, earlier: Self) -> Option<u64> {
        self.spent?.checked_sub(earlier.spent?)
    }
}

/// The actual request admits encoding work and observes allocation safe points.
/// The canonical encoder owns checked sizes and fallible scratch allocation.
pub(super) struct EncodingMeter<'work> {
    work: &'work mut PreparationWork,
    control: &'work mut dyn FnMut(CanonicalEncodingCharge) -> Result<(), CheckpointDenial>,
}

impl EncodingMeter<'_> {
    pub(super) fn admit(
        &mut self,
        charge: CanonicalEncodingCharge,
    ) -> Result<(), CheckpointDenial> {
        (self.control)(charge)?;
        match charge {
            CanonicalEncodingCharge::Work(work) => self
                .work
                .spend(Some(work))
                .map_err(CheckpointDenial::Resource),
            CanonicalEncodingCharge::Scratch(_) => Ok(()),
        }
    }
}

/// Derive one native identity under the same request that owns preparation.
/// A serializer cannot turn a swallowed admission stop into successful identity.
pub(super) fn encode_identity<T, Stopped>(
    execution: &QueryRequestExecution<'_>,
    work: &mut PreparationWork,
    encode: impl FnOnce(
        &mut EncodingMeter<'_>,
    )
        -> Result<T, ApplicationComputationPartitionIdentityDenial<CheckpointDenial>>,
    encode_polled: impl FnOnce(
        &mut dyn FnMut() -> Result<(), CheckpointDenial>,
    ) -> Result<
        T,
        ApplicationComputationPartitionIdentityDenial<CheckpointDenial>,
    >,
    value_denial: impl FnOnce(ApplicationValueEncodeDenial) -> Denial<Stopped>,
) -> Result<T, Denial<Stopped>> {
    let encoded = if !execution.has_accounting() {
        execution.decision_stage(|context| {
            encode_polled(&mut || context.poll().map_err(checkpoint_denial))
        })
    } else {
        execution.decision_stage(|context| {
            let mut control = |charge| {
                match charge {
                    CanonicalEncodingCharge::Work(units) if context.has_accounting() => {
                        context.checkpoint(units)
                    }
                    CanonicalEncodingCharge::Work(_) => context.poll(),
                    CanonicalEncodingCharge::Scratch(_) => context.poll(),
                }
                .map_err(checkpoint_denial)
            };
            let mut meter = EncodingMeter {
                work,
                control: &mut control,
            };
            encode(&mut meter)
        })
    }
    .map_err(preparation_denial)?;
    encoded.map_err(|denial| match denial {
        ApplicationComputationPartitionIdentityDenial::Key(denial) => value_denial(denial),
        ApplicationComputationPartitionIdentityDenial::Admission(denial) => {
            checkpoint_failure(denial)
        }
        ApplicationComputationPartitionIdentityDenial::CapacityOverflow => {
            Denial::Resource(Resource::CapacityOverflow)
        }
        ApplicationComputationPartitionIdentityDenial::Allocation => {
            Denial::Resource(Resource::NativeAllocationRefused {
                kind: worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator,
                requested_payload_bytes: None,
            })
        }
    })
}

fn checkpoint_failure<Stopped>(denial: CheckpointDenial) -> Denial<Stopped> {
    match denial {
        CheckpointDenial::Resource(denial) => Denial::Resource(denial),
        CheckpointDenial::Interrupted(denial) => Denial::Interrupted(denial),
        CheckpointDenial::NestedPatternStopped => Denial::NestedPatternStopped,
    }
}

/// Admit fresh routing or carried canonical work through its original request.
/// Reader work and canonical encoding work keep their distinct native receipts.
pub(super) fn charge_preparation<Stopped>(
    execution: &QueryRequestExecution<'_>,
    work: &mut PreparationWork,
    units: Option<u64>,
) -> Result<(), Denial<Stopped>> {
    if !execution.has_accounting() {
        return execution.checkpoint().map_err(Denial::from_kernel_stop);
    }
    let units = units.ok_or(Denial::Resource(Resource::WorkCounterOverflow))?;
    execution
        .decision_stage(|context| context.checkpoint(units).map_err(checkpoint_denial))
        .map_err(preparation_denial)?
        .map_err(checkpoint_failure)?;
    work.spend(Some(units)).map_err(Denial::Resource)
}

fn preparation_denial<Stopped>(denial: WorthQueryAdvancementDenial) -> Denial<Stopped> {
    match denial {
        WorthQueryAdvancementDenial::Resource(denial) => Denial::Resource(denial),
        WorthQueryAdvancementDenial::Interrupted(denial) => Denial::Interrupted(denial),
        WorthQueryAdvancementDenial::NestedStopped => Denial::NestedPatternStopped,
        WorthQueryAdvancementDenial::NestedOpening => {
            Denial::Resource(Resource::NestedAdvancementOpening)
        }
        WorthQueryAdvancementDenial::Panicked => Denial::PreparationPanicked,
    }
}
