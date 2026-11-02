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
        self.attach_declarations(&actual.declarations)
    }
    fn attach_declarations(
        &mut self,
        actual: &DurableKindAspectDeclarations,
    ) -> Result<(), &'static str> {
        if actual.identity_declarations.len() != self.identities.len()
            || actual.merge_policy_declarations.len() != self.policies.len()
        {
            return Err("durable declaration leaf admission count mismatch");
        }
        for (memory, declaration) in self
            .identities
            .iter_mut()
            .zip(&actual.identity_declarations)
        {
            memory.attach(&declaration.basis)?;
        }
        for (memory, declaration) in self
            .policies
            .iter_mut()
            .zip(&actual.merge_policy_declarations)
        {
            crate::merge::data::attach_policy_name(memory, &declaration.policy)?;
        }
        Ok(())
    }
}

impl DurableBranchRootSchemaImage {
    pub(crate) fn decode_control_bytes() -> Option<usize> {
        std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<SchemaImageData>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
    }
    pub(crate) fn decode_relation_roster_bytes(capacity: usize) -> Option<usize> {
        std::alloc::Layout::array::<DurableRelationSchemaRegistration>(capacity)
            .ok()
            .map(|layout| layout.size())
    }
    pub(crate) fn retain_decode_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        declarations: &mut [DurableSchemaDeclarationMemory],
    ) -> Result<(), &'static str> {
        let actual = std::sync::Arc::get_mut(&mut self.0)
            .ok_or("schema image shared before decode admission")?;
        if actual._memory.is_some()
            || declarations.len()
                != actual.entity_registrations.len() + actual.relation_registrations.len()
        {
            return Err("schema image decode admission mismatch");
        }
        for (entry, grant) in actual
            .entity_registrations
            .iter()
            .map(|entry| &entry.declarations)
            .chain(
                actual
                    .relation_registrations
                    .iter()
                    .map(|entry| &entry.declarations),
            )
            .zip(declarations)
        {
            grant.attach_declarations(entry)?;
        }
        actual._memory = memory;
        Ok(())
    }
}
