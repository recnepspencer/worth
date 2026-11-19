//! Preparation reads the Source rows Native observed changing the receiving
//! invocation's scope, through Native's own admitted managed read.
use super::{
    WorthQueryApplicationInvariantEntity as Entity,
    WorthQueryApplicationInvariantReadView as ReadView,
    WorthQueryApplicationInvariantScopePlanner as Planner,
    WorthQueryPreparedInvariantManagedIndex as Prepared,
};
use crate::domain_computation::primary_graph::application_invariant::{
    WorthQueryApplicationInvariantEntityBinding as EntityBinding,
    WorthQueryApplicationInvariantError as Error, WorthQueryInvariantAccessDenial as Access,
    WorthQueryInvariantAccessDenialKind as AccessKind,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        ManagedIndexSourcePreparationDenial as NativeDenial, ManagedIndexSourceProgram,
        PreparedManagedIndexSourceRead,
    },
};

impl<Schema> Planner<'_, '_, Schema> {
    /// The rows of `entity`'s kind Native's own scope closure observed changing
    /// this invocation's scope that are committed members of it, typed for
    /// the committed view: rows the proposal retired or changed.
    pub fn committed_observed_source_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        self.typed_rows(prepared, entity, self.committed(), true)
    }
    /// The rows of `entity`'s kind Native's own scope closure observed changing
    /// this invocation's scope that are proposed members of it, typed for
    /// the proposed view: rows the proposal introduced or changed. A
    /// replacement under a retired row's key is another entity.
    pub fn proposed_observed_source_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        self.typed_rows(prepared, entity, self.proposed(), false)
    }
    fn typed_rows<I, E, Cause>(
        &self,
        prepared: &Prepared<Schema, I>,
        entity: &EntityBinding<Schema, E>,
        view: ReadView<'_, Schema>,
        before: bool,
    ) -> Result<Vec<Entity<Schema, E>>, Error<Cause>> {
        let (binding, program, native) = prepared.source();
        let request = Request {
            planner: &self.binding_identity,
            index: binding,
            entity: &entity.binding_identity,
            kind: entity.entity_kind,
            program,
            native,
            before,
            element_bytes: size_of::<Entity<Schema, E>>(),
        };
        let rows = observed_row_ids(self, request).map_err(|cause| match cause {
            Denied::Native(cause) => Error::from(cause),
            Denied::Access(cause) => Error::from(cause),
        })?;
        Ok(rows
            .into_iter()
            .map(|row| view.entity(row, entity.entity_kind))
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

/// The one concrete core: exact bindings, Native's admitted read of this
/// invocation and observation, membership-qualified rows, result storage
/// claimed before it grows, and each row admitted to the prepared scope.
/// Cost: one membership read and one admission per observed row.
fn observed_row_ids<Schema>(
    planner: &Planner<'_, '_, Schema>,
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
    let source = planner
        .lower
        .read_managed_index_source(request.native)
        .map_err(Denied::Native)?;
    let members = source
        .changed_occurrences_in(request.before)
        .map_err(Denied::Native)?;
    let relations = planner.lower.relations();
    let bytes = members
        .len()
        .checked_mul(request.element_bytes + size_of::<EntityId>())
        .ok_or_else(|| {
            Denied::Access(Access::new(AccessKind::WorkBudgetExceeded, "observed rows"))
        })?;
    relations
        .read_budget()
        .claim_retained(bytes as u64)
        .map_err(|cause| Denied::Access(Access::from_native_read(cause, "observed rows")))?;
    let mut rows = Vec::with_capacity(members.len());
    for member in members {
        planner.reads.endpoint();
        planner
            .admission
            .entity(member.entity(), planner.lower.relations().read_budget())
            .map_err(Denied::Access)?;
        rows.push(member.entity());
    }
    Ok(rows)
}
