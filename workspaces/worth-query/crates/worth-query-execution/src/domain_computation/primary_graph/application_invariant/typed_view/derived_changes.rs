//! A typed rule scope retains Native's genuine paired candidate change proof.
use super::{
    WorthQueryApplicationInvariantContext as Context,
    WorthQueryApplicationInvariantScopePlanner as Planner,
};
use crate::domain_computation::primary_graph::application_invariant::{
    WorthQueryApplicationInvariantDerivedRelationBinding as Binding,
    WorthQueryInvariantAccessDenial as Denial, WorthQueryInvariantAccessDenialKind as Kind,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    runtime::{DerivedRelationChangeData, PreparedDerivedRelationChanges},
    schema::{DerivedRelationHandle, NativeMemberKey},
};

/// Owned source custody for one installed recipe in one actual mutation.
/// Store this in the rule's Scope; it grants no row or mutation authority.
pub struct WorthQueryPreparedInvariantDerivedChanges<Schema, Relation, From, To> {
    source: PreparedDerivedRelationChanges,
    binding: ApplicationSchemaBindingIdentity,
    definition: DerivedRelationHandle,
    marker: PhantomData<fn() -> (Schema, Relation, From, To)>,
}

/// Descriptive structural pullback lent only after exact Native re-admission.
/// Projected scalar changes require their own per-entry dependency routing.
pub struct WorthQueryInvariantDerivedChanges<'source> {
    data: &'source DerivedRelationChangeData,
}
impl WorthQueryInvariantDerivedChanges<'_> {
    pub fn affected_roots(&self) -> &std::collections::BTreeSet<NativeMemberKey> {
        self.data.affected_roots()
    }
    pub fn affected_targets(&self) -> &std::collections::BTreeSet<NativeMemberKey> {
        self.data.affected_targets()
    }
    /// Empty inverse-prefix support remains part of the retained Native proof.
    pub fn support(&self) -> &[worth_relational::facade::runtime::DerivedRelationData] {
        self.data.support()
    }
}

impl<Schema> Planner<'_, '_, Schema> {
    pub fn prepare_derived_relation_changes<Relation, From, To>(
        &self,
        binding: &Binding<Schema, Relation, From, To>,
    ) -> Result<WorthQueryPreparedInvariantDerivedChanges<Schema, Relation, From, To>, Denial> {
        if binding.binding_identity != self.binding_identity {
            return Err(Denial::new(Kind::ForeignBinding, "candidate change source"));
        }
        let source = self
            .lower
            .prepare_derived_relation_changes(&binding.definition)
            .map_err(super::derived_relation::native_denial)?;
        Ok(WorthQueryPreparedInvariantDerivedChanges {
            source,
            binding: self.binding_identity.clone(),
            definition: binding.definition.clone(),
            marker: PhantomData,
        })
    }
}

impl<Schema> Context<'_, '_, Schema> {
    pub fn derived_relation_changes<'source, Relation, From, To>(
        &self,
        binding: &Binding<Schema, Relation, From, To>,
        prepared: &'source WorthQueryPreparedInvariantDerivedChanges<Schema, Relation, From, To>,
    ) -> Result<WorthQueryInvariantDerivedChanges<'source>, Denial> {
        if binding.binding_identity != self.binding_identity
            || prepared.binding != self.binding_identity
            || prepared.definition != binding.definition
        {
            return Err(Denial::new(Kind::ForeignBinding, "candidate change source"));
        }
        let data = self
            .lower
            .read_prepared_derived_relation_changes(&prepared.source)
            .map_err(super::derived_relation::native_denial)?;
        Ok(WorthQueryInvariantDerivedChanges { data })
    }
}
