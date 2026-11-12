//! Portable provider meaning; executable projections belong to their runtimes.
use crate::boundary_protocol::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BridgeConditionalProviderRole {
    Condition,
    DependencyComparator,
    OutputComparator,
    ArtifactReuseComparator,
    Trigger,
    Wake,
    Compute,
}
impl BridgeConditionalProviderRole {
    pub const ALL: [Self; 7] = [
        Self::Condition,
        Self::DependencyComparator,
        Self::OutputComparator,
        Self::ArtifactReuseComparator,
        Self::Trigger,
        Self::Wake,
        Self::Compute,
    ];
}

/// Complete owner-defined selection and parameters, independent of a build.
/// No executable object or process-local type identity is carried here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeAuthoredProviderDescription {
    protocol: Arc<BoundaryProtocolIdentity>,
    version: BoundaryProtocolVersion,
    payload: Arc<[u8]>,
}
impl BridgeAuthoredProviderDescription {
    pub fn new(
        protocol: BoundaryProtocolIdentity,
        version: BoundaryProtocolVersion,
        payload: impl Into<Arc<[u8]>>,
    ) -> Self {
        Self {
            protocol: Arc::new(protocol),
            version,
            payload: payload.into(),
        }
    }
    pub fn protocol(&self) -> &BoundaryProtocolIdentity {
        &self.protocol
    }
    pub fn version(&self) -> BoundaryProtocolVersion {
        self.version
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Named slots cannot duplicate a role or confuse its provider contract.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BridgeAuthoredProviderDescriptions {
    pub condition: Option<BridgeAuthoredProviderDescription>,
    pub dependency_comparator: Option<BridgeAuthoredProviderDescription>,
    pub output_comparator: Option<BridgeAuthoredProviderDescription>,
    pub reuse_comparator: Option<BridgeAuthoredProviderDescription>,
    pub trigger: Option<BridgeAuthoredProviderDescription>,
    pub wake: Option<BridgeAuthoredProviderDescription>,
    pub compute: Option<BridgeAuthoredProviderDescription>,
}
impl BridgeAuthoredProviderDescriptions {
    pub fn get(
        &self,
        role: BridgeConditionalProviderRole,
    ) -> Option<&BridgeAuthoredProviderDescription> {
        match role {
            BridgeConditionalProviderRole::Condition => self.condition.as_ref(),
            BridgeConditionalProviderRole::DependencyComparator => {
                self.dependency_comparator.as_ref()
            }
            BridgeConditionalProviderRole::OutputComparator => self.output_comparator.as_ref(),
            BridgeConditionalProviderRole::ArtifactReuseComparator => {
                self.reuse_comparator.as_ref()
            }
            BridgeConditionalProviderRole::Trigger => self.trigger.as_ref(),
            BridgeConditionalProviderRole::Wake => self.wake.as_ref(),
            BridgeConditionalProviderRole::Compute => self.compute.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredProviderReconstructionDenial {
    ForeignProtocol,
    UnsupportedVersion(crate::boundary_protocol::BoundaryProtocolUnsupportedVersion),
    MalformedPayload,
    FreshBindingUnavailable,
}
impl BridgeAuthoredProviderDescription {
    pub fn admit_source_version(
        &self,
        candidate: &Self,
    ) -> Result<(), AuthoredProviderReconstructionDenial> {
        if self.protocol() != candidate.protocol() {
            return Err(AuthoredProviderReconstructionDenial::ForeignProtocol);
        }
        crate::boundary_protocol::BoundaryProtocolCompatibilityWindow::inclusive(
            self.version(),
            self.version(),
        )
        .admit(candidate.version())
        .map_err(AuthoredProviderReconstructionDenial::UnsupportedVersion)?;
        Ok(())
    }
}
