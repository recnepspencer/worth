//! Frozen exact Native proofs include empty physical adjacency closure.
use super::*;
use worth_relational::facade::runtime::{
    DerivedRelationReadIdentity, PreparedDerivedRelationRead, StructuralRelationView,
};
use worth_relational::facade::{identity::EntityId, schema::DerivedRelationHandle};

impl ApplicationInvariantAdmissionAccess {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn derived(
        &self,
        key: DerivedRelationReadIdentity,
        view: &StructuralRelationView<'_>,
        definition: &DerivedRelationHandle,
        root: EntityId,
    ) -> Result<(PreparedDerivedRelationRead, bool), WorthQueryInvariantAccessDenial> {
        let budget = view.read_budget();
        budget.check_live().map_err(retention_denial)?;
        match self {
            Self::Planning(admission) => {
                let mut admission = admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned");
                if let Some(prepared) = admission.derived.get(&key) {
                    return Ok((prepared.clone(), false));
                }
                let prepared = view
                    .prepare_derived_relations_from(definition, root)
                    .map_err(super::super::typed_view::derived_relation::native_denial)?;
                budget
                    .claim_retained(super::retention_extent::derived_entry_bytes())
                    .map_err(retention_denial)?;
                admission.derived.insert(key, prepared.clone());
                Ok((prepared, true))
            }
            Self::Evaluation(admission) => admission
                .derived
                .get(&key)
                .cloned()
                .map(|prepared| (prepared, false))
                .ok_or_else(|| denial("derived relation source")),
        }
    }
}
