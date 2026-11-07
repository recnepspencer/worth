//! The non-cloneable runtime root retains the sole canonical owner partition.
use super::readmission::SignalOwnerSourceReadmission;
use super::{
    SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor, SignalDurableHomeBinding,
    SignalDurableMedium, SignalOwnerReadmissionDenial,
};
use crate::branch::owner_services::durable_publication::{
    SignalDurableBranchMutationPort, SignalDurablePublicationState,
};
use crate::branch::owner_services::SignalDurableBranchSourcePort;
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::logic::transaction::SignalRuntime;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalConditionalDefinitionOccurrence,
};

/// A Home occurrence paired with the contract installed in its actual recovered graph.
pub struct SignalReadmittedConditionalContract {
    occurrence: DurableSignalConditionalDefinitionOccurrence,
    installed: InstalledSignalConditionalContract,
}
impl SignalReadmittedConditionalContract {
    pub(crate) fn new(
        occurrence: DurableSignalConditionalDefinitionOccurrence,
        installed: InstalledSignalConditionalContract,
    ) -> Self {
        Self {
            occurrence,
            installed,
        }
    }
    pub const fn occurrence(&self) -> DurableSignalConditionalDefinitionOccurrence {
        self.occurrence
    }
    pub fn installed(&self) -> &InstalledSignalConditionalContract {
        &self.installed
    }
}

/// No raw runtime, mutable graph, or memory-only movement service can escape
/// this root. Its weak source service reads the actual sealed branch cells.
pub struct SignalDurableOwner<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    _runtime: Box<SignalRuntime<D, I, E, Ctx, T>>,
    publication: Arc<SignalDurablePublicationState<D, I, T>>,
    port: SignalDurableBranchSourcePort<D, I, T>,
    contracts: BTreeMap<DurableSignalBranchIdentity, Vec<SignalReadmittedConditionalContract>>,
}
impl<D, I, E, Ctx, T> SignalDurableOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn from_readmitted(
        runtime: Box<SignalRuntime<D, I, E, Ctx, T>>,
        source: SignalOwnerSourceReadmission,
        medium: Arc<dyn SignalDurableMedium>,
        owner: std::sync::Weak<crate::branch::owner_services::SignalOwner<D, I, T>>,
        vocabulary: Arc<super::SignalDurableTierVocabulary<T>>,
        contracts: BTreeMap<DurableSignalBranchIdentity, Vec<SignalReadmittedConditionalContract>>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, SignalOwnerReadmissionDenial> {
        let port = SignalDurableBranchSourcePort::new(
            owner.clone(),
            medium.binding(),
            Arc::clone(&vocabulary),
        );
        let publication = SignalDurablePublicationState::new(
            owner,
            medium,
            vocabulary,
            source.into_locations(execution)?,
        );
        Ok(Self {
            _runtime: runtime,
            publication,
            port,
            contracts,
        })
    }
    pub fn binding(&self) -> &SignalDurableHomeBinding {
        self.publication.binding()
    }
    pub fn source_port(&self) -> SignalDurableBranchSourcePort<D, I, T> {
        self.port.clone()
    }
    pub fn take_installation_contracts(
        &mut self,
    ) -> BTreeMap<DurableSignalBranchIdentity, Vec<SignalReadmittedConditionalContract>> {
        std::mem::take(&mut self.contracts)
    }
    /// Read an exact original source record, including historical snapshot source.
    /// The descriptive artifact grants no live or historical branch admission.
    pub fn read_definition_source(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDefinitionTransitionArtifact, SignalOwnerReadmissionDenial> {
        if descriptor.owner() != self.binding().owner() {
            return Err(SignalOwnerReadmissionDenial::ForeignOwner);
        }
        self.publication
            .read_definition_source(descriptor, execution)
    }
    pub fn read_original_advance(
        &self,
        original: worth_foundational::facade::SignalBranchAdvanceIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<super::SignalBranchAdvanceArtifact, SignalOwnerReadmissionDenial> {
        self.publication.read_original_advance(original, execution)
    }
    pub fn mutation_port(&self) -> SignalDurableBranchMutationPort<D, I, E, Ctx, T> {
        SignalDurableBranchMutationPort::from_owner(&self.publication)
    }
}
