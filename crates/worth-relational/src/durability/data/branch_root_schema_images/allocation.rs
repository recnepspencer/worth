//! Concrete backing owned by the private durable declaration roster.
use super::*;
impl DurableEntitySchemaRegistration {
    /// Capacities, not entry counts: the decoder supplies its checked Vec
    /// growth/overlap bounds. Nested strings/contracts remain separately quoted.
    pub(crate) fn declaration_roster_storage_bytes(
        aspect_capacity: usize,
        identity_capacity: usize,
        policy_capacity: usize,
    ) -> Option<usize> {
        std::alloc::Layout::array::<DurableDeclaredAspectBinding>(aspect_capacity)
            .ok()?
            .size()
            .checked_add(
                std::alloc::Layout::array::<IdentityBasisDeclaration>(identity_capacity)
                    .ok()?
                    .size(),
            )?
            .checked_add(
                std::alloc::Layout::array::<AspectMergePolicyDeclaration>(policy_capacity)
                    .ok()?
                    .size(),
            )
    }
}

/// One decode's admissions for independently shared identity/policy leaves.
pub(crate) struct DurableSchemaDeclarationMemory {
    pub(crate) identities: Vec<crate::merge::data::MergeIdentityMemory>,
    pub(crate) policies: Vec<Option<crate::merge::data::MergeLeafMemory>>,
    pub(crate) _tree_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl DurableSchemaDeclarationMemory {
    pub(crate) fn attach(
        &mut self,
        actual: &DurableEntitySchemaRegistration,
    ) -> Result<(), &'static str> {
        if actual.declarations.identity_declarations.len() != self.identities.len()
            || actual.declarations.merge_policy_declarations.len() != self.policies.len()
        {
            return Err("durable declaration leaf admission count mismatch");
        }
        for (memory, declaration) in self
            .identities
            .iter_mut()
            .zip(&actual.declarations.identity_declarations)
        {
            memory.attach(&declaration.basis)?;
        }
        for (memory, declaration) in self
            .policies
            .iter_mut()
            .zip(&actual.declarations.merge_policy_declarations)
        {
            crate::merge::data::attach_policy_name(memory, &declaration.policy)?;
        }
        Ok(())
    }
}
