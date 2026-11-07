use std::any::TypeId;
use std::collections::{BTreeSet, HashMap};

use worth_relational::facade::runtime::CustomInvariantRegistration;

use crate::application::WorthQueryDomainEntryMarker;
use crate::runtime::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
    WorthQueryGraphReadRegistryAdmissionError,
};

use super::{
    assemble_installed_domain_artifact, classify_pending_package, compile_package_invariants,
    lower_package_substrates, WorthQueryAdmittedDomainPackage, WorthQueryCompiledDomainSubstrates,
    WorthQueryDomainInstallationDenial, WorthQueryDomainInstallationDenialKind,
    WorthQueryInstalledDomainArtifact, WorthQueryLoweredPackageSubstrates,
    WorthQueryPendingPackageCandidate,
};

#[derive(Default)]
pub(crate) struct WorthQueryPendingDomainInstallations {
    artifacts: Vec<WorthQueryInstalledDomainArtifact>,
    compiled_substrates: WorthQueryCompiledDomainSubstrates,
    marker_types: HashMap<TypeId, String>,
    package_identities: BTreeSet<String>,
    domain_owners: BTreeSet<String>,
    invariant_slots: BTreeSet<String>,
    declaration_family_slots: BTreeSet<String>,
}

#[cfg(test)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryPendingDomainInstallationSnapshot {
    artifact_packages: Vec<String>,
    marker_packages: Vec<String>,
    package_identities: Vec<String>,
    domain_owners: Vec<String>,
    invariant_slots: Vec<String>,
    declaration_family_slots: Vec<String>,
    compiled_invariants: Vec<String>,
}

impl WorthQueryPendingDomainInstallations {
    #[cfg(test)]
    pub(crate) fn certification_snapshot(&self) -> WorthQueryPendingDomainInstallationSnapshot {
        let mut artifact_packages = self
            .artifacts
            .iter()
            .map(|artifact| artifact.package_identity.as_str().to_string())
            .collect::<Vec<_>>();
        artifact_packages.sort();
        let mut marker_packages = self.marker_types.values().cloned().collect::<Vec<_>>();
        marker_packages.sort();
        let mut compiled_invariants = self
            .compiled_substrates
            .custom_invariants
            .iter()
            .map(|registration| registration.rule_id().as_str().to_string())
            .collect::<Vec<_>>();
        compiled_invariants.sort();
        WorthQueryPendingDomainInstallationSnapshot {
            artifact_packages,
            marker_packages,
            package_identities: self.package_identities.iter().cloned().collect(),
            domain_owners: self.domain_owners.iter().cloned().collect(),
            invariant_slots: self.invariant_slots.iter().cloned().collect(),
            declaration_family_slots: self.declaration_family_slots.iter().cloned().collect(),
            compiled_invariants,
        }
    }

    pub(crate) fn install<D>(
        &mut self,
        package: WorthQueryAdmittedDomainPackage<D>,
    ) -> Result<(), WorthQueryDomainInstallationDenial>
    where
        D: WorthQueryDomainEntryMarker + 'static,
    {
        let candidate = classify_pending_package(&package)?;
        self.reject_identity_conflicts(
            candidate.marker_type,
            &candidate.package_identity,
            &candidate.domain_owner,
        )?;
        self.reject_new_slots(
            &candidate.invariant_slots,
            &self.invariant_slots,
            WorthQueryDomainInstallationDenialKind::ConflictingInvariant,
        )?;
        self.reject_new_slots(
            &candidate.declaration_family_slots,
            &self.declaration_family_slots,
            WorthQueryDomainInstallationDenialKind::ConflictingDeclarationFamily,
        )?;
        let lowered_substrates = lower_package_substrates(&package, &candidate);
        self.validate_operation_union(&lowered_substrates.graph_read_operations)?;
        let custom_invariants = compile_package_invariants(&package, &candidate)?;
        let WorthQueryLoweredPackageSubstrates {
            graph_read_operations,
        } = lowered_substrates;
        let artifact =
            assemble_installed_domain_artifact(package, &candidate, graph_read_operations);
        self.publish_installation(candidate, custom_invariants, artifact);
        Ok(())
    }

    fn publish_installation(
        &mut self,
        candidate: WorthQueryPendingPackageCandidate,
        custom_invariants: Vec<CustomInvariantRegistration>,
        artifact: WorthQueryInstalledDomainArtifact,
    ) {
        self.marker_types
            .insert(candidate.marker_type, candidate.package_identity.clone());
        self.package_identities.insert(candidate.package_identity);
        self.domain_owners.insert(candidate.domain_owner);
        self.invariant_slots.extend(candidate.invariant_slots);
        self.declaration_family_slots
            .extend(candidate.declaration_family_slots);
        self.artifacts.push(artifact);
        self.compiled_substrates
            .custom_invariants
            .extend(custom_invariants);
    }

    pub(crate) fn take_compiled_substrates(&mut self) -> WorthQueryCompiledDomainSubstrates {
        std::mem::take(&mut self.compiled_substrates)
    }

    pub(crate) fn compiled_substrates_are_empty(&self) -> bool {
        self.compiled_substrates.custom_invariants.is_empty()
    }

    pub(crate) fn host_installation_packages(
        &self,
    ) -> Vec<worth_query_installation::facade::WorthQueryAdmittedPortableDomainPackage> {
        let mut packages = self
            .artifacts
            .iter()
            .map(|artifact| artifact.portable_package.clone())
            .collect::<Vec<_>>();
        packages.sort_by(|left, right| left.package().identity().cmp(right.package().identity()));
        packages
    }

    pub(crate) fn into_artifacts(mut self) -> Vec<WorthQueryInstalledDomainArtifact> {
        self.artifacts.sort_by(|left, right| {
            left.package_identity
                .as_str()
                .cmp(right.package_identity.as_str())
        });
        self.artifacts
    }

    fn reject_identity_conflicts(
        &self,
        marker_type: TypeId,
        package_identity: &str,
        domain_owner: &str,
    ) -> Result<(), WorthQueryDomainInstallationDenial> {
        if self.marker_types.contains_key(&marker_type) {
            return Err(WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::DuplicateMarkerType,
                domain_owner,
            ));
        }
        if self.package_identities.contains(package_identity) {
            return Err(WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::DuplicatePackageIdentity,
                package_identity,
            ));
        }
        if self.domain_owners.contains(domain_owner) {
            return Err(WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::ConflictingDomainOwner,
                domain_owner,
            ));
        }
        Ok(())
    }

    fn reject_new_slots(
        &self,
        new_slots: &[String],
        existing_slots: &BTreeSet<String>,
        kind: WorthQueryDomainInstallationDenialKind,
    ) -> Result<(), WorthQueryDomainInstallationDenial> {
        if let Some(conflict) = new_slots.iter().find(|slot| existing_slots.contains(*slot)) {
            return Err(WorthQueryDomainInstallationDenial::new(kind, conflict));
        }
        Ok(())
    }

    fn validate_operation_union(
        &self,
        additions: &[WorthQueryGraphReadOperationRegistration],
    ) -> Result<(), WorthQueryDomainInstallationDenial> {
        let registrations = self
            .artifacts
            .iter()
            .flat_map(|artifact| artifact.graph_read_operations.iter().cloned())
            .chain(additions.iter().cloned())
            .collect::<Vec<_>>();
        WorthQueryGraphReadOperationRegistry::admit(registrations)
            .map(|_| ())
            .map_err(operation_admission_denial)
    }
}

fn operation_admission_denial(
    error: WorthQueryGraphReadRegistryAdmissionError,
) -> WorthQueryDomainInstallationDenial {
    let kind = match error {
        WorthQueryGraphReadRegistryAdmissionError::AmbiguousDomainReferenceAdmission => {
            WorthQueryDomainInstallationDenialKind::AmbiguousGraphReadRelationScope
        }
        _ => WorthQueryDomainInstallationDenialKind::ConflictingGraphReadOperation,
    };
    WorthQueryDomainInstallationDenial::new(kind, error.as_str())
}
