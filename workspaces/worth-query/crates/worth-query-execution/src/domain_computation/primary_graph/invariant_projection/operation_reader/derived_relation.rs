//! Derived reads issue the complete physical dependency closure, not cached authority.
use super::super::{
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use super::decision_reads::DecisionReadOutcome;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationFactKey as Fact,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget as Target;
use worth_relational::facade::{
    runtime::{DerivedRelationReadDenial as NativeDenial, PinnedDerivedRelationRead},
    schema::DerivedRelationDenial as DefinitionDenial,
    transactions::RelationalExecutionDenialCause as Cause,
};

mod member;
mod source;
pub use member::{
    WorthQueryInvariantDerivedMemberRelation, WorthQueryInvariantDerivedWalk,
    WorthQueryInvariantDerivedWalkStep, WorthQueryInvariantGraphComponent,
    WorthQueryInvariantGraphMember,
};

/// One canonical derived endpoint pair. It has no stored relation identity.
pub struct WorthQueryInvariantDerivedRelation<Schema, Relation, From, To> {
    from: WorthQueryInvariantEntityIdentity<Schema, From>,
    to: WorthQueryInvariantEntityIdentity<Schema, To>,
    marker: PhantomData<fn() -> Relation>,
}
impl<S, R, F, T> WorthQueryInvariantDerivedRelation<S, R, F, T> {
    pub fn from(&self) -> &WorthQueryInvariantEntityIdentity<S, F> {
        &self.from
    }
    pub fn to(&self) -> &WorthQueryInvariantEntityIdentity<S, T> {
        &self.to
    }
}

impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn decision_derived_count<
        Relation: OperationReads<Operation>,
        From,
        To,
    >(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
        association:&worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
    ) -> Result<u64, Denial> {
        let result = self.read_derived(relation, root).and_then(|native| {
            let count = u64::try_from(native.data().endpoints().len()).map_err(|_| {
                self.reader
                    .walk_stop(Cause::WorkCounterOverflow, relation.name())
            })?;
            let product = self
                .selected_product
                .ok_or_else(|| Denial::new(Kind::UndeclaredDecisionTarget, relation.name()))?;
            let definition = &self
                .reader
                .layout
                .derived_relation(relation.name())
                .expect("read admitted this installed definition")
                .handle;
            let observed = self.reader.managed_views.retain_native_count(
                self.runtime_authority.as_u64(),
                self.binding_identity,
                definition,
                association,
                root.entity_id,
                product,
                native.data(),
                self.reader.execution,
            );
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(observed.unwrap_or(count))
        });
        if result.is_err() {
            self.fail_derived(root, relation.from());
        }
        result
    }

    /// Read a declared derived source and retain every physical and empty read
    /// as this operation's decision facts. Cost follows the explored recipe paths.
    pub fn decision_derived_relations_from<Relation: OperationReads<Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<Vec<WorthQueryInvariantDerivedRelation<Schema, Relation, From, To>>, Denial> {
        let result = self.read_derived(relation, root).and_then(|native| {
            if !matches!(relation.to_member(), worth_query_declaration::facade::application_schema::ApplicationGraphMemberSelector::Entity(_)) {
                return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
            }
            let target_kind = self
                .reader
                .layout
                .entity_kind(relation.to())
                .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
            let mut pairs = Vec::new();
            pairs
                .try_reserve_exact(native.data().endpoints().len())
                .map_err(|_| {
                    self.reader
                        .walk_stop(Cause::AllocationFailed, relation.name())
                })?;
            for endpoint in native.data().endpoints() {
                self.reader
                    .execution
                    .check_live()
                    .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
                pairs.push(WorthQueryInvariantDerivedRelation {
                    from: WorthQueryInvariantEntityIdentity {
                        entity_id: root.entity_id,
                        kind: root.kind,
                        entity: Arc::clone(&root.entity),
                        authority_identity: root.authority_identity,
                        _marker: PhantomData,
                    },
                    to: WorthQueryInvariantEntityIdentity {
                        entity_id: endpoint.entity().ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))?,
                        kind: target_kind,
                        entity: Arc::from(relation.to()),
                        authority_identity: self.reader.authority_identity,
                        _marker: PhantomData,
                    },
                    marker: PhantomData,
                });
            }
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(pairs)
        });
        if result.is_err() {
            self.fail_derived(root, relation.from());
        }
        result
    }
}
