use std::sync::Arc;

use worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution;

use super::{
    AuthoritativeConditionalInstallation, ConditionalOperationKey,
    WorthQueryConditionalExecutionRegistry, WorthQueryInstalledConditionalNode,
};
use crate::domain_installation::{
    WorthQueryConditionalNodeInstallationDenial as Denial, WorthQueryDomainInstallationRegistry,
};

impl WorthQueryConditionalExecutionRegistry {
    pub(crate) fn prepare_runtime_reconstitution(
        &self,
        domains: &WorthQueryDomainInstallationRegistry,
        candidate: &BridgePreparedConditionalReconstitution,
    ) -> Result<Self, Denial> {
        let authoritative = self
            .authoritative
            .iter()
            .map(|installed| readmit_installation(installed, domains, candidate))
            .collect::<Result<_, _>>()?;
        let mut rebuilt = Self {
            authoritative,
            by_operation: Default::default(),
        };
        rebuilt.by_operation = rebuilt.rebuilt_index();
        Ok(rebuilt)
    }
}

fn readmit_installation(
    installed: &AuthoritativeConditionalInstallation,
    domains: &WorthQueryDomainInstallationRegistry,
    candidate: &BridgePreparedConditionalReconstitution,
) -> Result<AuthoritativeConditionalInstallation, Denial> {
    let ConditionalOperationKey {
        domain,
        operation,
        family,
    } = installed.key;
    let domain = domains
        .authority_by_marker(domain)
        .ok_or(Denial::DomainNotInstalled)?;
    let operation = domains
        .execution_index()
        .domain_operation_authority(installed.key.domain, operation, family)
        .ok_or(Denial::OperationNotInstalled)?;
    let node = &installed.node;
    let source = &operation.authority;
    if node.runtime_authority != domain.runtime_authority().as_u64()
        || node.operation_identity != source.definition().canonical_identity()
        || node.installation_runtime_authority != source.runtime_ordinal()
        || node.source_installation_generation != source.generation().ordinal()
        || super::super::installation::declared_node(source.definition(), &node.location)
            != Some(&node.declaration)
    {
        return Err(Denial::DeclarationLookupDrift);
    }
    let lowering = candidate
        .readmit_lowering(&node.lowering)
        .map_err(|denial| Denial::Bridge {
            kind: denial.kind(),
            detail: denial.detail().to_owned(),
        })?;
    Ok(AuthoritativeConditionalInstallation {
        key: installed.key,
        node: Arc::new(WorthQueryInstalledConditionalNode {
            lowering,
            location: node.location.clone(),
            declaration: node.declaration.clone(),
            graph_authority: Arc::clone(&node.graph_authority),
            operation_identity: node.operation_identity.clone(),
            runtime_authority: node.runtime_authority,
            installation_runtime_authority: node.installation_runtime_authority,
            source_installation_generation: node.source_installation_generation,
            installation_generation: domain.installation_generation().ordinal(),
            resource_support: node.resource_support.clone(),
        }),
    })
}
