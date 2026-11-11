//! Exact application kind lookup at the installed layout owner.
use worth_relational::facade::identity::KindId;
impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn application_relation_kinds(
        &self,
    ) -> impl Iterator<Item = KindId> + '_ {
        self.physical_roles.relation_ids.iter().copied()
    }

    pub(in crate::domain_computation::primary_graph) fn application_entity_kinds(
        &self,
    ) -> impl Iterator<Item = KindId> + '_ {
        self.physical_roles.entity_ids.iter().copied()
    }

    pub(in crate::domain_computation) fn is_application_entity_kind(&self, kind: KindId) -> bool {
        self.physical_roles.entity_ids.contains(&kind)
    }

    pub(in crate::domain_computation) fn is_application_relation_kind(&self, kind: KindId) -> bool {
        self.physical_roles.relation_ids.contains(&kind)
    }

    pub(in crate::domain_computation) fn entity_kind(&self, entity: &str) -> Option<KindId> {
        self.physical_roles.entities.get(entity).copied()
    }

    pub(in crate::domain_computation::primary_graph) fn entity_kind_lookup_work(
        &self,
        entity: &str,
    ) -> Option<u64> {
        let entries = self.physical_roles.entities.len();
        let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
        let comparisons = entries.min(11).checked_mul(levels)?.checked_add(1)?;
        let bytes_per_comparison = entity.len().checked_add(1)?;
        u64::try_from(comparisons.checked_mul(bytes_per_comparison)?).ok()
    }

    pub fn entity_name(&self, kind: KindId) -> Option<&str> {
        self.physical_roles
            .entities
            .iter()
            .find_map(|(name, candidate)| (*candidate == kind).then_some(name.as_str()))
    }
}

impl super::WorthQueryPrimaryGraphLayout {
    /// Resolve actual create-effect kind custody without inventing a name.
    /// Each visited integer kind is funded before comparison; names are borrowed.
    pub(in crate::domain_computation::primary_graph) fn record_name_admitted<E>(
        &self,
        owner: worth_query_installation::facade::ApplicationFieldBindingOwnerKind,
        kind: KindId,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<Option<&str>, E> {
        use worth_query_installation::facade::ApplicationFieldBindingOwnerKind as Owner;
        match owner {
            Owner::Entity => {
                for (name, candidate) in &self.physical_roles.entities {
                    admit(1)?;
                    if *candidate == kind {
                        return Ok(Some(name));
                    }
                }
            }
            Owner::Relation => {
                for (name, candidate) in &self.relation_kinds {
                    admit(1)?;
                    if candidate.kind == kind {
                        return Ok(Some(name));
                    }
                }
            }
        }
        Ok(None)
    }
}

mod court_doors_for_test;
