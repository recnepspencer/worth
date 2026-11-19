//! Preparation reads the Source rows Native observed changing the receiving
//! invocation's scope, through Native's own admitted managed read.
use super::super::prepared_scope::ApplicationInvariantAdmissionAccess as Admission;
use super::super::read_observation::InvariantReadObservation as Reads;
use super::{
    ViewPosture, WorthQueryApplicationInvariantEntity as Entity,
    WorthQueryApplicationInvariantScopePlanner as Planner,
    WorthQueryPreparedInvariantManagedIndex as Prepared,
};
use crate::domain_computation::primary_graph::application_invariant::{
    WorthQueryApplicationInvariantEntityBinding as EntityBinding,
    WorthQueryApplicationInvariantError as Error, WorthQueryInvariantAccessDenial as Access,
    WorthQueryInvariantAccessDenialKind as AccessKind,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        CustomInvariantScopePlanner, ManagedIndexSourcePreparationDenial as NativeDenial,
        ManagedIndexSourceProgram, PreparedManagedIndexSourceRead,
    },
};

impl<Schema> Planner<'_, '_, Schema> {
    /// The committed side of the Source's exact delta for this invocation's
    /// scope: rows whose materialized entry the proposal removed or changed,
    /// as the provider owner's maintenance found them in the logical
    /// observation, typed for the committed view. Preparation only, through
    /// this invocation's own admitted Source read.
    pub fn committed_observed_source_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        self.typed_rows(prepared, entity, ViewPosture::Committed)
    }
    /// The proposed side of the same exact delta: rows whose materialized
    /// entry the proposal added or changed, typed for the proposed view. A
    /// scope born after an ordered predecessor, warm, cold or seeded, lends
    /// its delta relative to the predecessor's membership in the proposed
    /// view, never an inherited unchanged row; a newly built scope with no
    /// ordered predecessor lends every member. A replacement under a retired
    /// row's key is another entity.
    pub fn proposed_observed_source_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        self.typed_rows(prepared, entity, ViewPosture::Proposed)
    }
    /// Type-dependent extraction around the concrete core, then handles
    /// minted for the admitted rows only.
    fn typed_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
        posture: ViewPosture,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        let (binding, program, native) = prepared.source();
        let request = Request {
            planner: &self.binding_identity,
            index: binding,
            entity: &entity.binding_identity,
            kind: entity.entity_kind,
            program,
            native,
            before: matches!(posture, ViewPosture::Committed),
            element_bytes: size_of::<Entity<Schema, E>>(),
        };
        let rows = observed_row_ids(self.lower, &self.admission, &self.reads, request).map_err(
            |cause| match cause {
                Denied::Native(cause) => Error::from(cause),
                Denied::Access(cause) => Error::from(cause),
            },
        )?;
        let affinity = self.lower.proposal_affinity();
        Ok(rows
            .into_iter()
            .map(|entity_id| Entity {
                entity_id,
                kind: entity.entity_kind,
                binding_identity: self.binding_identity.clone(),
                proposal_affinity: affinity,
                posture,
                _marker: PhantomData,
            })
            .collect())
    }
}

/// The erased inputs of one observed-row read.
struct Request<'a> {
    planner: &'a ApplicationSchemaBindingIdentity,
    index: &'a ApplicationSchemaBindingIdentity,
    entity: &'a ApplicationSchemaBindingIdentity,
    kind: KindId,
    program: &'a ManagedIndexSourceProgram,
    native: &'a PreparedManagedIndexSourceRead,
    before: bool,
    element_bytes: usize,
}
enum Denied {
    Native(NativeDenial),
    Access(Access),
}

/// The one concrete core, compiled once: exact bindings, Native's admitted
/// read of this invocation and observation, the owner's recorded delta,
/// result storage claimed before it grows, and each row admitted to the
/// prepared scope. Cost: one admission per lent row.
fn observed_row_ids(
    lower: &CustomInvariantScopePlanner<'_>,
    admission: &Admission,
    reads: &Reads,
    request: Request<'_>,
) -> Result<Vec<EntityId>, Denied> {
    let foreign = |subject| Denied::Access(Access::new(AccessKind::ForeignBinding, subject));
    if request.index != request.planner {
        return Err(foreign("managed index binding"));
    }
    if request.entity != request.planner {
        return Err(foreign("Source row binding"));
    }
    if request.kind != request.program.row_kind() {
        return Err(Denied::Access(Access::new(
            AccessKind::WrongEntityKind,
            "Source row kind",
        )));
    }
    let relations = lower.relations();
    lower
        .lend_managed_index_source_changes(request.native, request.before, |members| {
            let bytes = members
                .len()
                .checked_mul(request.element_bytes + size_of::<EntityId>())
                .ok_or_else(|| {
                    Denied::Access(Access::new(AccessKind::WorkBudgetExceeded, "observed rows"))
                })?;
            relations
                .read_budget()
                .claim_retained(bytes as u64)
                .map_err(|cause| {
                    Denied::Access(Access::from_native_read(cause, "observed rows"))
                })?;
            let mut rows = Vec::with_capacity(members.len());
            for &row in members {
                reads.endpoint();
                admission
                    .entity(row, relations.read_budget())
                    .map_err(Denied::Access)?;
                rows.push(row);
            }
            Ok(rows)
        })
        .map_err(Denied::Native)?
}
