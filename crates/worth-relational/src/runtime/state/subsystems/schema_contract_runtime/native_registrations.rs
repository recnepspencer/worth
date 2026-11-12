//! Immutable native rules keep their own original compilation backing.
use super::SchemaContractRuntimeSubsystem;
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::identity::data::KindId;
use crate::schema::{data::*, SchemaCompilationControl};
use crate::validation::data::InvariantRegistration;
use std::{alloc::Layout, ops::Deref, sync::Arc};
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug, Default)]
pub(crate) struct NativeRelationIntegrityInventory {
    registrations: Vec<InvariantRegistration>,
    #[cfg(test)]
    plans: RelationIntegrityPlanCatalog,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Deref for NativeRelationIntegrityInventory {
    type Target = [InvariantRegistration];
    fn deref(&self) -> &Self::Target {
        &self.registrations
    }
}
impl NativeRelationIntegrityInventory {
    pub(super) fn compile(
        registry: &RelationalSchemaRegistry,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let quote = NativeRegistrationQuote::measure(registry, control)?;
        let memory = control.reserve(quote.retained)?;
        let temporary = control.reserve(quote.temporary)?;
        let inventory = Self::compile_owned(registry, control, quote.registrations, memory)?;
        drop(temporary);
        Ok(inventory)
    }
    fn compile_owned(
        registry: &RelationalSchemaRegistry,
        control: SchemaCompilationControl<'_, '_>,
        capacity: usize,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<Self>, Denial> {
        control.checkpoint()?;
        let plans =
            crate::schema::lower_relation_integrity_plans_for_compilation(registry, control)?;
        let mut registrations = Vec::with_capacity(capacity);
        for plan in plans.relation_plans.values() {
            control.checkpoint()?;
            crate::validation::data::append_relation_integrity_registrations(
                plan,
                &mut registrations,
                control,
            )?;
        }
        control.checkpoint()?;
        let inventory = Arc::new(Self {
            registrations,
            #[cfg(test)]
            plans,
            _memory: memory,
        });
        #[cfg(not(test))]
        drop(plans);
        Ok(inventory)
    }
    #[cfg(test)]
    pub(crate) fn plan(&self, kind: &KindId) -> Option<&LoweredRelationIntegrityPlan> {
        self.plans.relation_plans.get(kind)
    }
}
struct NativeRegistrationQuote {
    registrations: usize,
    retained: usize,
    temporary: usize,
}
impl NativeRegistrationQuote {
    fn measure(
        registry: &RelationalSchemaRegistry,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut plan = usize::try_from(
            crate::branch::retained_compilation_tree_bytes::<KindId, LoweredRelationIntegrityPlan>(
                registry.relation_kinds.len(),
            )
            .map_err(|_| Denial::MemoryOverflow)?,
        )
        .map_err(|_| Denial::MemoryOverflow)?;
        let mut retained = arc::<NativeRelationIntegrityInventory>()?;
        add(&mut retained, arc::<SchemaContractRuntimeSubsystem>()?)?;
        let mut temporary = 0usize;
        let mut registrations = 0usize;
        for relation in registry.relation_kinds.values() {
            control.checkpoint()?;
            let quote = relation.relation_integrity.lowering_backing(control)?;
            add(&mut registrations, quote.registrations)?;
            add(&mut retained, quote.rules)?;
            add(&mut plan, quote.plan)?;
            add(&mut temporary, quote.scratch)?;
        }
        add(
            &mut retained,
            registrations
                .checked_mul(size_of::<InvariantRegistration>())
                .ok_or(Denial::MemoryOverflow)?,
        )?;
        #[cfg(test)]
        add(&mut retained, plan)?;
        #[cfg(not(test))]
        add(&mut temporary, plan)?;
        Ok(Self {
            registrations,
            retained,
            temporary,
        })
    }
}
fn add(total: &mut usize, bytes: usize) -> Result<(), Denial> {
    *total = total.checked_add(bytes).ok_or(Denial::MemoryOverflow)?;
    Ok(())
}
fn arc<T>() -> Result<usize, Denial> {
    arc_layout(Layout::new::<T>())
}
fn arc_layout(data: Layout) -> Result<usize, Denial> {
    Ok(Layout::new::<[usize; 2]>()
        .extend(data)
        .map_err(|_| Denial::MemoryOverflow)?
        .0
        .pad_to_align()
        .size())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
    use worth_execution::SerialMemoryBudget;

    #[test]
    fn cloned_native_rule_inventories_share_the_original_compiler_charge_without_copying_rules() {
        let control = SchemaCompilationControl::Configuration;
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "native-rule-member".into(),
                schema_id: SchemaId("native-rule".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap()
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "native-rule-edge".into(),
                schema_id: SchemaId("native-rule".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::Forbid,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "native-rule-endpoint".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: true,
                        cross_context_policy: CrossContextPolicy::Forbid,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
            .unwrap();
        let quote = NativeRegistrationQuote::measure(&registry, control).unwrap();
        let memory = SerialMemoryBudget::new(None);
        let retained = memory.reserve(quote.retained as u64).unwrap();
        let temporary = memory.reserve(quote.temporary as u64).unwrap();
        let rules = NativeRelationIntegrityInventory::compile_owned(
            &registry,
            control,
            quote.registrations,
            Some(retained),
        )
        .unwrap();
        drop(temporary);
        let charged = memory.charged_memory_bytes();
        assert!(charged > 0);
        assert!(!rules.is_empty());
        assert!(rules.plan(&KindId(2)).is_some());
        let fork = rules.clone();
        assert!(Arc::ptr_eq(&rules, &fork));
        drop(rules);
        assert_eq!(memory.charged_memory_bytes(), charged);
        assert!(!fork.is_empty());
        drop(fork);
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
}
