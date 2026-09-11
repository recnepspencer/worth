use std::{collections::BTreeMap, sync::Arc};

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ErasedApplicationCapabilityContract},
    application_schema::{
        ApplicationOperationProgramTarget, ApplicationOperationRef,
        ApplicationSchemaBindingIdentity, ApplicationSchemaMember,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

use crate::{
    authority_cryptography::AuthoritySeal, installed_index::WorthQueryInstalledPackageAuthority,
};

use super::{
    authority_seal::derive_capability_authority_seal, canonical_basis::prepare_capability_basis,
    delegation::CompiledApplicationCapabilityDelegation,
    WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationCapabilityInstallationDenialKind, WorthQueryCapabilityCanonicalArtifact,
    WorthQueryInstalledApplicationCapabilityIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ApplicationCapabilityRegistryKey {
    name: String,
    capability_type: WorthQueryPortableTypeIdentity,
    operation: String,
    operation_type: WorthQueryPortableTypeIdentity,
    input_type: WorthQueryPortableTypeIdentity,
}

impl ApplicationCapabilityRegistryKey {
    pub(crate) fn from_contract(contract: &ErasedApplicationCapabilityContract) -> Self {
        Self {
            name: contract.name().to_string(),
            capability_type: contract.capability_identity(),
            operation: contract.operation().to_string(),
            operation_type: contract.operation_identity(),
            input_type: contract.input_identity(),
        }
    }

    pub(crate) fn from_references<Schema, Capability, Operation, Input>(
        capability: &ApplicationCapabilityRef<Schema, Capability>,
        operation: &ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Self {
        Self {
            name: capability.name().to_string(),
            capability_type: capability.marker_identity(),
            operation: operation.name().to_string(),
            operation_type: WorthQueryPortableTypeIdentity::declared(operation.name()),
            input_type: operation.input_identity(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

pub(crate) struct CompiledApplicationCapability {
    canonical: WorthQueryCapabilityCanonicalArtifact,
    identity: WorthQueryInstalledApplicationCapabilityIdentity,
    authority_identity: AuthoritySeal,
    contract: ErasedApplicationCapabilityContract,
    delegation: CompiledApplicationCapabilityDelegation,
}

impl CompiledApplicationCapability {
    pub(crate) fn canonical(&self) -> &WorthQueryCapabilityCanonicalArtifact {
        &self.canonical
    }

    pub(crate) fn identity(&self) -> &WorthQueryInstalledApplicationCapabilityIdentity {
        &self.identity
    }

    pub(crate) fn authority_identity(&self) -> &AuthoritySeal {
        &self.authority_identity
    }

    pub(crate) fn contract(&self) -> &ErasedApplicationCapabilityContract {
        &self.contract
    }

    pub(crate) fn delegation_activation_program(
        &self,
    ) -> Option<&[ApplicationOperationProgramTarget]> {
        self.delegation.activation_program()
    }
}

pub(crate) fn compile_capability_registry(
    package: &WorthQueryInstalledPackageAuthority,
    binding: &ApplicationSchemaBindingIdentity,
    members: &[ApplicationSchemaMember],
) -> Result<
    BTreeMap<ApplicationCapabilityRegistryKey, Arc<CompiledApplicationCapability>>,
    WorthQueryApplicationCapabilityInstallationDenial,
> {
    let mut registry = BTreeMap::new();
    for contract in members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract),
        _ => None,
    }) {
        let canonical = prepare_capability_basis(
            binding.package_identity(),
            binding.schema_identity(),
            contract,
        )?;
        let identity =
            WorthQueryInstalledApplicationCapabilityIdentity::from_canonical(*canonical.digest());
        let authority_identity = derive_capability_authority_seal(
            &package.authority_key,
            binding,
            identity.bytes(),
            contract,
        );
        let key = ApplicationCapabilityRegistryKey::from_contract(contract);
        let delegation = CompiledApplicationCapabilityDelegation::compile(contract);
        let compiled = Arc::new(CompiledApplicationCapability {
            canonical,
            identity,
            authority_identity,
            contract: contract.clone(),
            delegation,
        });
        if registry.insert(key, compiled).is_some() {
            return Err(WorthQueryApplicationCapabilityInstallationDenial::new(
                WorthQueryApplicationCapabilityInstallationDenialKind::CapabilityMeaningChanged,
                contract.name(),
            ));
        }
    }
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;

    struct Schema;
    struct Operation;
    worth_query_declaration::worth_query_structured_value_binding!(
        OperationInputBinding for () { identity: "worth.rust.unit" }
    );

    worth_query_declaration::worth_query_capability!(
        Capability in Schema,
        identity "worth.query.test.capability.v1"
    );

    impl ApplicationOperationMarkerIdentity<Schema> for Operation {
        type InputBinding = OperationInputBinding;
        const IDENTIFIER: &'static str = "RetainedOperation";
    }

    #[test]
    fn warm_registry_lookup_uses_retained_portable_marker_identities() {
        let capability = ApplicationCapabilityRef::<Schema, Capability>::from_declaration();
        let operation = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration();
        let key = ApplicationCapabilityRegistryKey::from_references(&capability, &operation);

        assert_eq!(
            key.capability_type.as_str(),
            "worth.query.test.capability.v1"
        );
        assert_eq!(key.operation_type.as_str(), "RetainedOperation");
        assert_eq!(key.input_type.as_str(), "worth.rust.unit");
    }
}
