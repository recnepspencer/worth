//! Immutable installed Source requirements, independent of rule execution.
use super::ManagedIndexSourceProgram;
use crate::schema::SchemaCompilationControl;
use crate::validation::data::{CustomInvariantAccessContract, CustomInvariantRegistration};
use crate::validation::CustomInvariantCompilationDenial as CompilationDenial;
use crate::{derived_relation::DerivedRelationDenial, schema::data::RelationalSchemaRegistry};
use std::{num::NonZeroU64, sync::Arc};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourcePreparationGroup {
    programs: Arc<[ManagedIndexSourceProgram]>,
    access: CustomInvariantAccessContract,
    maximum_work_units: Option<NonZeroU64>,
    owner: PreparationOwner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreparationOwner {
    Invariant,
    Public,
}

impl SourcePreparationGroup {
    /// Freeze only actual requirements. Relative rule order remains the Source
    /// owner order; rules without programs cannot contribute a posting or slot.
    pub(crate) fn from_invariant(
        registration: &CustomInvariantRegistration,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Option<Self>, CompilationDenial> {
        control.checkpoint().map_err(CompilationDenial::Execution)?;
        if registration.managed_sources.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            programs: registration.managed_sources.clone(),
            access: registration.access_contract().clone(),
            maximum_work_units: registration.maximum_work_units(),
            owner: PreparationOwner::Invariant,
        }))
    }

    /// The installed program supplies the entire audience. No caller access or
    /// new work allowance may widen this original-request preparation group.
    pub(crate) fn for_public_sources(
        registry: &RelationalSchemaRegistry,
        programs: Vec<ManagedIndexSourceProgram>,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Vec<Self>, CompilationDenial> {
        let mut groups: Vec<Self> = Vec::with_capacity(programs.len());
        for program in programs {
            control.checkpoint().map_err(CompilationDenial::Execution)?;
            program.require_registry(registry, control)?;
            for group in &groups {
                control.checkpoint().map_err(CompilationDenial::Execution)?;
                if group.programs.contains(&program) {
                    return Err(CompilationDenial::Source(
                        DerivedRelationDenial::DuplicateDefinition(
                            program.0.path.parts().identity.clone(),
                        ),
                    ));
                }
            }
            let mut leaves = 0usize;
            for _ in program.physical_sources() {
                control.checkpoint().map_err(CompilationDenial::Execution)?;
                leaves = leaves
                    .checked_add(1)
                    .ok_or(CompilationDenial::CapacityOverflow)?;
            }
            let entities_capacity = leaves
                .checked_mul(2)
                .and_then(|n| n.checked_add(2))
                .ok_or(CompilationDenial::CapacityOverflow)?;
            let mut entities = Vec::with_capacity(entities_capacity);
            entities.extend([program.scope_kind(), program.row_kind()]);
            let mut relations = Vec::with_capacity(leaves);
            for &(relation, from, to) in program.physical_sources() {
                control.checkpoint().map_err(CompilationDenial::Execution)?;
                entities.extend([from, to]);
                relations.push(relation);
            }
            let access = CustomInvariantAccessContract {
                read_entity_kinds: entities.clone(),
                affected_entity_kinds: entities,
                read_relation_kinds: relations.clone(),
                affected_relation_kinds: relations,
                include_relation_endpoint_entity_touches: false,
            }
            .canonicalize_for_compilation(control)
            .map_err(CompilationDenial::Execution)?;
            groups.push(Self {
                programs: Arc::from([program]),
                access,
                maximum_work_units: None,
                owner: PreparationOwner::Public,
            });
        }
        Ok(groups)
    }

    pub(crate) fn public_programs(&self) -> &[ManagedIndexSourceProgram] {
        match self.owner {
            PreparationOwner::Public => &self.programs,
            PreparationOwner::Invariant => &[],
        }
    }

    pub(crate) fn programs(&self) -> &[ManagedIndexSourceProgram] {
        &self.programs
    }

    pub(crate) fn access_contract(&self) -> &CustomInvariantAccessContract {
        &self.access
    }

    pub(crate) fn maximum_work_units(&self) -> Option<NonZeroU64> {
        self.maximum_work_units
    }
}
