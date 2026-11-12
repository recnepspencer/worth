//! Ordinary memory writers and authored owner services are disjoint installations.
use super::RuntimeWorldAuthoredSignalAttachment;
use std::sync::Arc;
use worth_signal::facade::branch::{
    SignalBranchBasisPort, SignalConditionalDefinitionPublicationPort, SignalOwnerServicePorts,
};
pub(crate) enum RuntimeWorldSignalServices<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    Memory {
        services: SignalOwnerServicePorts<D, I, E, Ctx, T>,
        definitions: SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
    },
    Authored(Arc<RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>>),
}
impl<D, I, E, Ctx, T> RuntimeWorldSignalServices<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) fn basis_port(&self) -> SignalBranchBasisPort<D, I, T> {
        match self {
            Self::Memory { services, .. } => services.basis_port(),
            Self::Authored(attachment) => attachment.basis_port(),
        }
    }
    pub(super) fn authored(
        &self,
    ) -> Option<&Arc<RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>>> {
        match self {
            Self::Memory { .. } => None,
            Self::Authored(attachment) => Some(attachment),
        }
    }
    pub(super) fn memory_services(&self) -> Option<&SignalOwnerServicePorts<D, I, E, Ctx, T>> {
        match self {
            Self::Memory { services, .. } => Some(services),
            Self::Authored(_) => None,
        }
    }
    pub(super) fn memory_definitions(
        &self,
    ) -> Option<&SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>> {
        match self {
            Self::Memory { definitions, .. } => Some(definitions),
            Self::Authored(_) => None,
        }
    }
}
