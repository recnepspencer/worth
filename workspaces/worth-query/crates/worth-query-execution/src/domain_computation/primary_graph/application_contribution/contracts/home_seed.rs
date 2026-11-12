//! Installation requirements come from declared contracts before any owner exists.
use super::WorthQueryApplicationContractCatalog;
use crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial;
use worth_foundational::facade::DurableStateFamily as Family;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationContractCatalog<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_home_seed(
        &self,
        declaration: &ErasedApplicationSchemaDeclaration,
        support: fn(Family, u8) -> Option<u8>,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        admit_declared_seed(
            !self.producers.is_empty(),
            !self.conditionals.is_empty(),
            declaration,
            support,
        )
    }
}

fn admit_declared_seed(
    has_producers: bool,
    has_conditionals: bool,
    declaration: &ErasedApplicationSchemaDeclaration,
    support: fn(Family, u8) -> Option<u8>,
) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
    let requirements = [
        (has_producers, Family::SignalNodeDefinitions),
        // Installed producers block Home retirement until producer custody exists,
        // so a Directory Home with producers would open but never close.
        (has_producers, Family::ProducerCleanupCustody),
        (has_conditionals, Family::BridgeConditionalDefinitions),
        (
            declaration.members().iter().any(requires_resource_nodes),
            Family::BridgeOwnedAsyncDefinitions,
        ),
    ];
    for (required, family) in requirements {
        if required && let Some(supported_from_phase) = support(family, 0) {
            return Err(WorthQueryDurableStateAdmissionDenial::Unsupported {
                family,
                supported_from_phase,
            });
        }
    }
    Ok(())
}

// Keep every schema member explicit: adding an installation input must revisit
// whether it allocates Signal nodes or Bridge correspondence at installation.
fn requires_resource_nodes(member: &ApplicationSchemaMember) -> bool {
    use ApplicationSchemaMember::*;
    match member {
        ApplicationResource { .. } => true,
        InvariantManagedIndexSource { .. }
        | ManagedIndex { .. }
        | ApplicationMutation { .. }
        | ApplicationInvariant { .. }
        | Entity { .. }
        | Aspect { .. }
        | Field { .. }
        | Relation { .. }
        | RelationAspect { .. }
        | DerivedRelation { .. }
        | RuleReadableCollection { .. }
        | OperationDerivedRead { .. }
        | ComponentSet { .. }
        | DerivedComponentSet { .. }
        | OperationComponentRead { .. }
        | PrincipalBinding { .. }
        | ApplicationQuery { .. }
        | ContentFormat { .. }
        | ApplicationHistory { .. }
        | ApplicationHistoryQuery { .. }
        | ApplicationQueryInspection { .. }
        | ApplicationCapability { .. }
        | ApplicationCapabilityContext { .. }
        | ApplicationCapabilityContextEntitySlot { .. }
        | ApplicationCapabilityProvenance { .. }
        | Operation { .. }
        | OperationProgram { .. }
        | OperationBody { .. }
        | OperationDecisionRead { .. }
        | DecisionReadScope { .. }
        | OperationDecisionReadScope { .. }
        | OperationMutationPrecondition { .. }
        | OperationProjectionWorkBudget { .. }
        | OperationExternalEffect { .. }
        | OperationInboundOccurrence { .. }
        | OperationAftermath { .. }
        | Policy { .. }
        | Ability { .. }
        | OperationAbility { .. }
        | AbilityPolicy { .. }
        | Unit { .. }
        | Effect { .. }
        | ProjectedMemberDomain { .. }
        | ScalarAspectField { .. } => false,
    }
}
