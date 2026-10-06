use super::QualifiedFilesystemMedia;

impl QualifiedFilesystemMedia {
    /// Derives the move-only Store durability admission basis from this exact
    /// qualified media generation and its sealed C.4 capability witness.
    #[cfg(feature = "store-runtime-owner")]
    pub fn physical_durability_admission_basis(
        &self,
    ) -> Result<crate::PhysicalDurabilityAdmissionBasis, crate::BackendCapabilityAdmissionDenial>
    {
        durability_admission_basis(
            self.store_identity(),
            self.basis().binding(),
            self.execution_capability(),
        )
    }

    #[cfg(feature = "store-runtime-owner")]
    pub fn physical_durability_admission_identity(
        &self,
    ) -> Result<crate::PhysicalDurabilityAdmissionIdentity, crate::BackendCapabilityAdmissionDenial>
    {
        Ok(self.physical_durability_admission_basis()?.identity())
    }
}

/// The one construction of a durability admission basis, shared by ordinary
/// open and recovery so both derive the same identity from the same media.
#[cfg(any(feature = "store-runtime-owner", feature = "recovery-runtime-owner"))]
pub(super) fn durability_admission_basis(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    binding: &super::qualification_basis::RootProfileBinding,
    capability: &crate::AdmittedBackendCapabilityWitness,
) -> Result<crate::PhysicalDurabilityAdmissionBasis, crate::BackendCapabilityAdmissionDenial> {
    let evidence = crate::CapabilityEvidenceClass::EstablishedByFilesystemAdmission;
    let file_sync = capability.require(crate::BackendCapabilityKind::Fsync, evidence)?;
    let directory_sync =
        capability.require(crate::BackendCapabilityKind::DirectorySync, evidence)?;
    let durable_rename =
        capability.require(crate::BackendCapabilityKind::DurableRename, evidence)?;
    Ok(
        crate::PhysicalDurabilityAdmissionBasis::from_qualified_media(
            crate::durability_profile::QualifiedDurabilityBasisInput {
                store,
                qualification_contract_version: binding.contract_version,
                root_identity: binding.root_identity,
                volume_identity: binding.volume_identity,
                profile_digest: binding.profile_digest,
                backend_build_identity: binding.backend_build_identity,
                target: capability.profile(),
                file_sync,
                directory_sync,
                durable_rename,
            },
        ),
    )
}
