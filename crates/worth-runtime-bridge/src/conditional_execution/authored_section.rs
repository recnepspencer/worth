//! Bridge's authored declaration section inside a Signal definition transition.
pub(super) mod branch_projection;
pub(crate) mod branch_registry;
pub(super) mod capture;
mod definition;
pub(super) mod definition_publication;
mod dependency;
mod fields;
mod fork_publication;
mod fork_section;
mod framing;
mod layout;
mod owned_async;
mod programme_affinity;
mod provider_source;
pub(super) mod publication;
pub(super) mod readmission;
mod registration;
mod rows;
mod targets;
pub use branch_projection::{
    BridgeAuthoredBranchDenial, BridgeAuthoredBranchView, BridgeReadmittedAuthoredAssembly,
};
pub(super) use definition::BridgeReadmittedDefinitionSource;
pub use definition_publication::{
    BridgeDurableDefinitionCompletion, BridgeDurableDefinitionCompletionDenial,
    BridgeDurableDefinitionPublicationDenial, BridgePreparedAuthoredDefinitionPublication,
};
pub use dependency::BridgeAuthoredDependencyDescription;
pub use fork_publication::BridgeDurableForkPublicationDenial;
pub use readmission::{BridgeAuthoredInstallationBindings, BridgeAuthoredReadmissionDenial};

use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, DurableSignalBranchIdentity,
};
use worth_signal::facade::durability::SignalBridgeAuthoredSection;

pub const BRIDGE_AUTHORED_SECTION_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.bridge.authored-section");
pub const BRIDGE_AUTHORED_SECTION_VERSION: BoundaryProtocolVersion =
    BoundaryProtocolVersion::new(3);

#[derive(Debug)]
pub enum BridgeAuthoredSectionDenial {
    Malformed,
    SizeLimit,
    ChecksumMismatch,
    ForeignBranch,
    UnsupportedVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    SignalCarrier(worth_signal::facade::durability::SignalDurableFrameDenial),
    SignalImage(worth_signal::facade::durability::SignalAuthoredImageDenial),
    SemanticContractEncoding,
    Provider(super::BridgeAuthoredProviderDenial),
    SignalDefinitionMismatch,
    SourceProfileRequired,
}

/// Descriptive authored bytes. Installation still needs fresh owner bindings.
#[derive(Clone, Debug)]
pub struct BridgeAuthoredSection {
    source: SignalBridgeAuthoredSection,
    branch: DurableSignalBranchIdentity,
    definition_count: usize,
}
impl BridgeAuthoredSection {
    pub fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub fn as_signal_section(&self) -> &SignalBridgeAuthoredSection {
        &self.source
    }
    pub fn into_signal_section(self) -> SignalBridgeAuthoredSection {
        self.source
    }
    pub fn readmit_bytes(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
        expected: DurableSignalBranchIdentity,
    ) -> Result<Self, BridgeAuthoredSectionDenial> {
        use BridgeAuthoredSectionDenial as D;
        framing::work(execution, bytes.len().saturating_mul(3))?;
        let parsed = layout::Layout::read(bytes)?;
        if parsed.branch != expected {
            return Err(D::ForeignBranch);
        }
        let _scratch = execution
            .reserve_retained_memory((bytes.len() as u64).checked_mul(2).ok_or(D::SizeLimit)?)
            .map_err(D::Request)?;
        parsed.inspect()?;
        let definition_count = rows::Rows::new(parsed.definitions)?.len();
        let source =
            SignalBridgeAuthoredSection::from_bridge_bytes(framing::copy(execution, bytes)?)
                .map_err(D::SignalCarrier)?;
        Ok(Self {
            source,
            branch: expected,
            definition_count,
        })
    }
    pub fn conditional_definition_count(&self) -> usize {
        self.definition_count
    }
}
