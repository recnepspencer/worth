//! Caller choices, resolved only against the private fresh adoption inventory.
use super::{
    AdmittedWorkflowDispositions, WorthQueryWorkflowAdoptionInventory,
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowInstanceDisposition,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
};

/// Explicit adoption intent, never an inventory, disclosure grant or prepared proof.
/// Each opaque reference names a fact the caller already possesses. Preparation
/// freshly checks every fact and requires a total legal decision. The explicit
/// carry policy applies to every fact found by that private safety check.
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdvancementPhase;
/// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
/// use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
/// fn advanced_cannot_disclose<S: ApplicationSchema>(selected: &WorthQuerySelectedProductOperation<'_, S>,
///     phase:&WorthQueryAdvancementPhase<'_>, target:&ApplicationProgramRevision, requirements:&WorthQueryProgramAdoptionRequirements) {
///     selected.workflow_adoption_inventory(phase,target,requirements,64);
/// }
/// ```
/// ```compile_fail,E0026
/// use worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;
/// fn no_payload(denial: WorthQueryBranchAdoptionPreparationDenial) {
///     if let WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired { inventory } = denial {
///         drop(inventory);
///     }
/// }
/// ```
/// ```compile_fail,E0532
/// use worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial as Denial;
/// fn no_private_commit_log(denial: Denial) {
///     match denial { Denial::RelationalPreparation(log) => drop(log), _ => {} }
/// }
/// ```
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial as Denial;
/// fn structural_category(denial: Denial) -> bool { matches!(denial,Denial::RelationalPreparation) }
/// ```
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryWorkflowDispositions;
/// fn explicit_intent() { let _ = WorthQueryWorkflowDispositions::carry_compatible(); }
/// ```
#[derive(Clone, Debug, Default)]
pub struct WorthQueryWorkflowDispositions {
    carry_compatible: bool,
    definitions: Vec<(
        PublishedWorkflowDefinitionRef,
        WorthQueryWorkflowDefinitionDisposition,
    )>,
    instances: Vec<(
        PublishedWorkflowInstanceRef,
        WorthQueryWorkflowInstanceDisposition,
    )>,
}
impl WorthQueryWorkflowDispositions {
    /// Begin explicit choices. An omitted live fact refuses preparation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Request carry for every fact that the native fresh safety inventory can
    /// legally carry. This performs no read and reveals no population.
    pub fn carry_compatible() -> Self {
        Self {
            carry_compatible: true,
            ..Self::default()
        }
    }
    /// Choose the disposition of one already possessed definition reference.
    #[must_use]
    pub fn definition(
        mut self,
        definition: &PublishedWorkflowDefinitionRef,
        disposition: WorthQueryWorkflowDefinitionDisposition,
    ) -> Self {
        self.definitions.push((definition.clone(), disposition));
        self
    }
    /// Choose the disposition of one already possessed instance reference.
    #[must_use]
    pub fn instance(
        mut self,
        instance: &PublishedWorkflowInstanceRef,
        disposition: WorthQueryWorkflowInstanceDisposition,
    ) -> Self {
        self.instances.push((instance.clone(), disposition));
        self
    }
    pub(in crate::domain_computation::primary_graph) fn resolve(
        &self,
        inventory: &WorthQueryWorkflowAdoptionInventory,
        branch: WorthQueryProductBranch,
    ) -> Result<AdmittedWorkflowDispositions, ()> {
        let mut choices = if self.carry_compatible {
            inventory.carry_compatible().map_err(|_| ())?
        } else {
            inventory.dispositions()
        };
        let mut definitions = std::collections::BTreeSet::new();
        let mut instances = std::collections::BTreeSet::new();
        for (reference, disposition) in &self.definitions {
            if reference.branch() != branch || !definitions.insert(reference.entity_id()) {
                return Err(());
            }
            let occurrence = inventory.definition(reference.entity_id()).ok_or(())?;
            choices = choices
                .definition(occurrence, *disposition)
                .map_err(|_| ())?;
        }
        for (reference, disposition) in &self.instances {
            if reference.branch() != branch || !instances.insert(reference.entity_id()) {
                return Err(());
            }
            let occurrence = inventory.instance(reference.entity_id()).ok_or(())?;
            choices = choices.instance(occurrence, *disposition).map_err(|_| ())?;
        }
        // Digest and complete legality remain native, never supplied by the caller.
        if choices.inventory_digest() != inventory.digest() {
            return Err(());
        }
        inventory.admit(&choices).map_err(|_| ())?;
        Ok(choices)
    }
}
