use std::sync::Arc;

use super::super::{
    BridgeCorrespondenceAdmissionIdentity, BridgeCorrespondenceBasis,
    BridgeInstalledSemanticCorrespondence, BridgeSemanticDependencyCandidate,
};
use worth_proof::AuthorityWitness;

use super::admission::BridgeRuntimeWorldAdmissionAuthorityMarker;

/// Owner-admitted Bridge meaning that may be carried into Runtime World.
///
/// The installed correspondence remains the Bridge authority. This wrapper
/// retains that witness and the exact configuration basis; it is not a second
/// mapping representation and cannot be constructed from a descriptor.
#[derive(Debug, Clone)]
pub struct AdmittedRuntimeWorldCorrespondenceBasis {
    basis: BridgeCorrespondenceBasis,
    dependency: Option<BridgeSemanticDependencyCandidate>,
    admission_identity: BridgeCorrespondenceAdmissionIdentity,
    _authority: Arc<AuthorityWitness<BridgeRuntimeWorldAdmissionAuthorityMarker>>,
}

impl PartialEq for AdmittedRuntimeWorldCorrespondenceBasis {
    fn eq(&self, other: &Self) -> bool {
        self.admission_identity == other.admission_identity
    }
}

impl Eq for AdmittedRuntimeWorldCorrespondenceBasis {}

impl AdmittedRuntimeWorldCorrespondenceBasis {
    pub(crate) fn from_installed(
        installed: &BridgeInstalledSemanticCorrespondence,
        authority: AuthorityWitness<BridgeRuntimeWorldAdmissionAuthorityMarker>,
    ) -> Self {
        Self {
            basis: installed.basis().clone(),
            dependency: Some(installed.dependency().clone()),
            admission_identity: installed.admission_identity().clone(),
            _authority: Arc::new(authority),
        }
    }

    pub(crate) fn baseline(
        runtime: &crate::facade::RuntimeBridge,
        signal_graph_instance_id: u64,
        authority: AuthorityWitness<BridgeRuntimeWorldAdmissionAuthorityMarker>,
    ) -> Self {
        let stable = Arc::<str>::from("bridge-runtime-baseline");
        Self {
            basis: BridgeCorrespondenceBasis {
                source_installation_identity: stable.clone(),
                source_basis: stable.clone(),
                source_runtime_authority: runtime.signal_runtime_key,
                source_installation_generation: 0,
                source_authority_binding_identity: stable.clone(),
                declared_graph_role: stable.clone(),
                graph_participation_identity: stable.clone(),
                graph_adapter_identity: stable,
                authoritative_source_profile: runtime.authoritative_source_profile.clone(),
                bridge_runtime_key: runtime.signal_runtime_key,
                signal_graph_instance_id,
                signal_partitions: Vec::new(),
            },
            dependency: None,
            admission_identity: BridgeCorrespondenceAdmissionIdentity::issue(),
            _authority: Arc::new(authority),
        }
    }

    pub fn basis(&self) -> &BridgeCorrespondenceBasis {
        &self.basis
    }

    /// Identity issued by Bridge for the installed correspondence admission.
    ///
    /// The identity binds the installed owner path for Runtime World
    /// composition; the descriptive correspondence basis is not the key.
    pub fn admission_identity(&self) -> &BridgeCorrespondenceAdmissionIdentity {
        &self.admission_identity
    }

    pub fn source_installation_generation(&self) -> u64 {
        self.basis.source_installation_generation()
    }

    pub fn signal_graph_instance_id(&self) -> u64 {
        self.basis.signal_graph_instance_id
    }

    pub(crate) fn dependency(&self) -> Option<&BridgeSemanticDependencyCandidate> {
        self.dependency.as_ref()
    }
}
