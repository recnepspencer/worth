//! Concrete backing owned by the private durable declaration roster.
use super::*;
/// One decode's admissions for independently shared identity/policy leaves.
pub struct DurableSchemaDeclarationMemory {
    pub identities:
        Vec<crate::schema::data::declaration_readmission::SchemaIdentityDeclarationMemory>,
    pub policies:
        Vec<Option<crate::schema::data::declaration_readmission::SchemaDeclarationLeafMemory>>,
    pub _tree_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl DurableSchemaDeclarationMemory {
    pub fn attach(&mut self, actual: &DurableEntitySchemaRegistration) -> Result<(), &'static str> {
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
            crate::schema::data::declaration_readmission::attach_policy_name(
                memory,
                &declaration.policy,
            )?;
        }
        Ok(())
    }
}

impl RecordedBranchSchemaImage {
    pub fn decode_control_bytes() -> Option<usize> {
        std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<SchemaImageData>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
    }
    pub fn decode_relation_roster_bytes(capacity: usize) -> Option<usize> {
        std::alloc::Layout::array::<DurableRelationSchemaRegistration>(capacity)
            .ok()
            .map(|layout| layout.size())
    }
    pub fn retain_decode_memory(
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
