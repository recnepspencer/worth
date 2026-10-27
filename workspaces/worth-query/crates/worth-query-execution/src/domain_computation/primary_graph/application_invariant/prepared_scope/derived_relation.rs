//! Frozen exact Native proofs include empty physical adjacency closure.
use super::*;
use worth_relational::facade::runtime::DerivedRelationReadShape;
use worth_relational::facade::runtime::{
    DerivedRelationReadIdentity, PreparedDerivedRelationRead, StructuralRelationView,
};
use worth_relational::facade::schema::{DerivedRelationHandle, NativeMemberKey};

impl ApplicationInvariantAdmissionAccess {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn derived(
        &self,
        key: DerivedRelationReadIdentity,
        view: &StructuralRelationView<'_>,
        definition: &DerivedRelationHandle,
        root: impl Into<NativeMemberKey>,
    ) -> Result<(PreparedDerivedRelationRead, bool), WorthQueryInvariantAccessDenial> {
        self.derived_source(
            key,
            view,
            definition,
            root.into(),
            DerivedRelationReadShape::Adjacency { reverse: false },
        )
    }

    pub(in crate::domain_computation::primary_graph::application_invariant) fn derived_source(
        &self,
        key: DerivedRelationReadIdentity,
        view: &StructuralRelationView<'_>,
        definition: &DerivedRelationHandle,
        root: NativeMemberKey,
        shape: DerivedRelationReadShape,
    ) -> Result<(PreparedDerivedRelationRead, bool), WorthQueryInvariantAccessDenial> {
        let budget = view.read_budget();
        budget.check_live().map_err(retention_denial)?;
        // Hash and full equality examine semantic projected names. The original
        // source meter pays for that real key work at every membership lookup.
        let bytes = match &root {
            NativeMemberKey::Entity(_) => 0,
            NativeMemberKey::Projected(member) => member
                .domain
                .schema
                .0
                .len()
                .checked_add(member.domain.name.len())
                .and_then(|n| n.checked_add(member.case.len()))
                .ok_or_else(|| {
                    WorthQueryInvariantAccessDenial::new(
                        WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded,
                        "derived member key size",
                    )
                })?,
        };
        match self {
            Self::Planning(admission) => {
                let mut admission = admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned");
                checkpoint_key(bytes, admission.derived.len(), budget)?;
                if let Some(prepared) = admission.derived.get(&key) {
                    return Ok((prepared.clone(), false));
                }
                let prepared = view
                    .prepare_derived_source(definition, root, shape)
                    .map_err(super::super::typed_view::derived_relation::native_denial)?;
                budget
                    .claim_retained(super::retention_extent::derived_entry_bytes())
                    .map_err(retention_denial)?;
                checkpoint_key(bytes, admission.derived.len(), budget)?;
                admission.derived.insert(key, prepared.clone());
                Ok((prepared, true))
            }
            Self::Evaluation(admission) => {
                checkpoint_key(bytes, admission.derived.len(), budget)?;
                admission
                    .derived
                    .get(&key)
                    .cloned()
                    .map(|prepared| (prepared, false))
                    .ok_or_else(|| denial("derived relation source"))
            }
        }
    }
}

fn checkpoint_key(
    bytes: usize,
    entries: usize,
    budget: CustomInvariantReadBudget<'_>,
) -> Result<(), WorthQueryInvariantAccessDenial> {
    // Hashing reads the semantic key once. Even a full-table collision search
    // cannot compare more than every retained key before lookup/insertion.
    let work = entries
        .checked_add(1)
        .and_then(|n| n.checked_mul(bytes))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| {
            WorthQueryInvariantAccessDenial::new(
                WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded,
                "derived key lookup",
            )
        })?;
    budget.checkpoint(work).map_err(retention_denial)
}
