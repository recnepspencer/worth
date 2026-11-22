//! Crate-boundary internal port for specialized effect program wrappers and
//! inbound admission denial construction. Never exported on
//! `worth-query-execution::facade` or Host.

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationEffectProgram, WorthQueryCapabilityRevocationProgram,
    WorthQueryDelegationActivationProgram, WorthQueryElevationApprovalProgram,
    WorthQueryElevationCloseProgram, WorthQueryElevationRequestProgram,
    WorthQueryMandatoryReviewProgram,
};
use crate::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundAdmissionDenial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;

pub trait SpecializedEffectProgramInternalPort<Schema, Operation, Input, Scope>: Sized {
    fn application_program(
        &self,
    ) -> &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>;

    fn into_inner(self) -> WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>;
}

macro_rules! impl_specialized_program_port {
    ($($ty:ident),* $(,)?) => {
        $(
            impl<Schema, Operation, Input, Scope>
                SpecializedEffectProgramInternalPort<Schema, Operation, Input, Scope>
                for $ty<Schema, Operation, Input, Scope>
            {
                fn application_program(
                    &self,
                ) -> &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
                    Self::application_program(self)
                }

                fn into_inner(
                    self,
                ) -> WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
                    Self::into_inner(self)
                }
            }
        )*
    };
}

impl_specialized_program_port!(
    WorthQueryCapabilityRevocationProgram,
    WorthQueryDelegationActivationProgram,
    WorthQueryElevationApprovalProgram,
    WorthQueryElevationCloseProgram,
    WorthQueryElevationRequestProgram,
    WorthQueryMandatoryReviewProgram,
);

pub trait InboundAdmissionDenialInternalPort: Sized {
    fn advancement(cause: WorthQueryAdvancementDenial) -> Self;
}

impl InboundAdmissionDenialInternalPort for WorthQueryInboundAdmissionDenial {
    fn advancement(cause: WorthQueryAdvancementDenial) -> Self {
        Self::advancement(cause)
    }
}
