use std::collections::BTreeSet;

use super::installation::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind, WorthQueryPendingConditionalOperation,
};
use super::lifecycle::WorthQueryConditionalOperationRegistry;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

pub(in crate::domain_computation::primary_graph) struct ConditionalRuntimeAffinity {
    runtime_authority: u64,
    installation_runtime: u64,
    installation_generation: u64,
    provider_identity: String,
    branch_identity: String,
}

impl ConditionalRuntimeAffinity {
    pub(super) fn bind(
        &self,
        identity: &super::canonical_identity::WorthQueryTemporalBindingIdentity,
    ) -> Result<
        super::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity,
        worth_foundational::facade::CanonicalDigestDerivationDenial,
    > {
        super::canonical_identity::prepare_temporal_runtime_binding_identity(
            super::canonical_identity::TemporalRuntimeBindingIdentityParts {
                binding: identity,
                runtime_authority: self.runtime_authority,
                installation_runtime: self.installation_runtime,
                installation_generation: self.installation_generation,
                provider: &self.provider_identity,
                branch: &self.branch_identity,
            },
        )
    }

    pub(super) fn runtime_authority(&self) -> u64 {
        self.runtime_authority
    }
}

pub(in crate::domain_computation::primary_graph) fn require_complete_binding_inventory<Schema>(
    expected: usize,
    bindings: &[Box<dyn WorthQueryPendingConditionalOperation<Schema>>],
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    let unique = bindings
        .iter()
        .map(|binding| binding.binding_identity())
        .collect::<BTreeSet<_>>();
    if bindings.len() == expected && unique.len() == expected {
        Ok(())
    } else {
        Err(WorthQueryConditionalRuntimeInstallationDenial::new(
            WorthQueryConditionalRuntimeInstallationDenialKind::IncompleteBindingInventory,
            format!(
                "expected {expected} exact conditional bindings, admitted {}",
                unique.len()
            ),
        ))
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph) fn install_pending_bindings<Schema>(
    bindings: Vec<Box<dyn WorthQueryPendingConditionalOperation<Schema>>>,
    bridge: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    authoritative_commit_cursor: u64,
    runtime_authority: u64,
    installation_runtime: u64,
    installation_generation: u64,
    provider_identity: &str,
    branch_identity: &str,
) -> Result<
    WorthQueryConditionalOperationRegistry<Schema>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let affinity = ConditionalRuntimeAffinity {
        runtime_authority,
        installation_runtime,
        installation_generation,
        provider_identity: provider_identity.to_string(),
        branch_identity: branch_identity.to_string(),
    };
    let mut registry = WorthQueryConditionalOperationRegistry::default();
    for binding in bindings {
        let installed = binding.install(bridge, graph, &affinity, authoritative_commit_cursor)?;
        registry.install(installed).map_err(|()| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::DuplicateBinding,
                "duplicate installed conditional binding",
            )
        })?;
    }
    Ok(registry)
}

pub(in crate::domain_computation::primary_graph) fn publication_denial(
    denial: WorthQueryPrimaryGraphInstallationDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::PrimaryGraphPublication,
        denial.to_string(),
    )
}
