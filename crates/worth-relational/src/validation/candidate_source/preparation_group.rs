//! Immutable installed Source requirements, independent of rule execution.
use super::ManagedIndexSourceProgram;
use crate::validation::data::{CustomInvariantAccessContract, CustomInvariantRegistration};
use crate::{derived_relation::DerivedRelationDenial, schema::data::RelationalSchemaRegistry};
use std::{num::NonZeroU64, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub(crate) fn from_invariant(registration: &CustomInvariantRegistration) -> Option<Self> {
        if registration.managed_sources.is_empty() {
            return None;
        }
        Some(Self {
            programs: registration.managed_sources.clone(),
            access: registration.access_contract().clone(),
            maximum_work_units: registration.maximum_work_units(),
            owner: PreparationOwner::Invariant,
        })
    }

    /// The installed program supplies the entire audience. No caller access or
    /// new work allowance may widen this original-request preparation group.
    pub(crate) fn for_public_sources(
        registry: &RelationalSchemaRegistry,
        programs: Vec<ManagedIndexSourceProgram>,
    ) -> Result<Vec<Self>, DerivedRelationDenial> {
        let mut groups: Vec<Self> = Vec::new();
        for program in programs {
            program.require_registry(registry)?;
            if groups.iter().any(|group| group.programs.contains(&program)) {
                return Err(DerivedRelationDenial::DuplicateDefinition(
                    program.0.path.parts().identity.clone(),
                ));
            }
            let mut entities = vec![program.scope_kind(), program.row_kind()];
            let mut relations = Vec::new();
            for &(relation, from, to) in program.physical_sources() {
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
            .canonicalize();
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
