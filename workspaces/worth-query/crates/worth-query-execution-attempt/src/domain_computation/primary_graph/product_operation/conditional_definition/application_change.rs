use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
};
use worth_runtime_world::facade::ProductBranchObservation;

/// A concrete conditional-definition successor admitted by the selected
/// application and product occurrence. The value is move-only and can enter
/// exactly one application effect program.
pub struct WorthQueryAdmittedApplicationConditionalDefinition {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    expected_product: ProductBranchObservation,
    identity: [u8; 32],
    successor: AdmittedSuccessor,
    bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    activations: Arc<
        crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
    >,
}

/// The Bridge successor at the selected product's Signal basis, or why that
/// basis serves no installed definition. A refusal is reported by the commit
/// only after its idempotency key resolves, so a retained original answers
/// its key before any Bridge basis check can refuse it.
pub type AdmittedSuccessor = Result<
    (
        Arc<BridgeInstalledConditionalLowering>,
        BridgeOwnedConditionalInstallationRequest,
    ),
    worth_runtime_bridge::facade::BridgeConditionalDenial,
>;

pub struct WorthQueryApplicationConditionalDefinitionParts {
    pub expected_product: ProductBranchObservation,
    pub predecessor: Arc<BridgeInstalledConditionalLowering>,
    pub request: BridgeOwnedConditionalInstallationRequest,
    pub bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    pub activations: Arc<
        crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
    >,
}

impl WorthQueryAdmittedApplicationConditionalDefinition {
    pub fn from_admitted_parts(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        expected_product: ProductBranchObservation,
        identity: [u8; 32],
        successor: AdmittedSuccessor,
        bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        activations: Arc<
            crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
        >,
    ) -> Self {
        Self {
            runtime_authority,
            application_binding,
            expected_product,
            identity,
            successor,
            bridge,
            activations,
        }
    }

    pub fn admits_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        application_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        product: &ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime_authority
            && &self.application_binding == application_binding
            && &self.expected_product == product
    }

    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    /// The commit's check, after its key resolves: the successor at the
    /// selected product's Signal basis, or why that basis serves none.
    pub fn check(
        self,
    ) -> Result<
        WorthQueryCheckedApplicationConditionalDefinition,
        worth_runtime_bridge::facade::BridgeConditionalDenial,
    > {
        let (predecessor, request) = self.successor?;
        Ok(WorthQueryCheckedApplicationConditionalDefinition {
            parts: WorthQueryApplicationConditionalDefinitionParts {
                expected_product: self.expected_product,
                predecessor,
                request,
                bridge: self.bridge,
                activations: self.activations,
            },
        })
    }
}

/// An admitted definition whose successor the commit found installed at its
/// product's Signal basis, after its key resolved. Only `check` mints it.
pub struct WorthQueryCheckedApplicationConditionalDefinition {
    parts: WorthQueryApplicationConditionalDefinitionParts,
}

impl WorthQueryCheckedApplicationConditionalDefinition {
    pub fn into_parts(self) -> WorthQueryApplicationConditionalDefinitionParts {
        self.parts
    }
}

impl std::fmt::Debug for WorthQueryCheckedApplicationConditionalDefinition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryCheckedApplicationConditionalDefinition")
            .field("expected_product", &self.parts.expected_product)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for WorthQueryAdmittedApplicationConditionalDefinition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAdmittedApplicationConditionalDefinition")
            .field("expected_product", &self.expected_product)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
