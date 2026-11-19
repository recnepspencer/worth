//! A declared index is borrowed through EACH actual Native observation.
use super::{WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantScopePlanner};
use crate::domain_computation::primary_graph::application_query::{
    managed_index::{
        invariant_read::{self, ManagedSourceValue},
        WorthQueryManagedIndexInvariantDenial, WorthQueryManagedIndexProbeValue,
    },
    WorthQueryApplicationProjection,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture as MatchPosture;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding},
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};
use worth_relational::facade::runtime::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
    PreparedManagedIndexSourceRead,
};

pub struct WorthQueryApplicationInvariantManagedIndexBinding<S, I> {
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) program: ManagedIndexSourceProgram,
    pub(in crate::domain_computation::primary_graph) marker: PhantomData<fn() -> (S, I)>,
}
impl<S, I> Clone for WorthQueryApplicationInvariantManagedIndexBinding<S, I> {
    fn clone(&self) -> Self {
        Self {
            binding_identity: self.binding_identity.clone(),
            program: self.program.clone(),
            marker: PhantomData,
        }
    }
}

/// Owned preparation data, without a borrowed execution request or read grant.
pub struct WorthQueryPreparedInvariantManagedIndex<S, I> {
    binding: WorthQueryApplicationInvariantManagedIndexBinding<S, I>,
    native: PreparedManagedIndexSourceRead,
}
impl<S, I> WorthQueryPreparedInvariantManagedIndex<S, I> {
    /// Describes the Native-discovered invocation. This identity admits no read.
    pub const fn scope_identity(&self) -> worth_relational::facade::identity::EntityId {
        self.native.scope()
    }
}

impl<Schema> super::WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// The Source rows of `entity`'s kind that Native observed changing the
    /// prepared index's scope, as this view admits them: Native's own scope
    /// closure, by entity identity, never a rule's walk of touched rows. A
    /// row retired by the proposal is absent from the proposed view and
    /// present in the committed one; projected field changes reach their root.
    /// Cost: O(observed rows) kind reads.
    pub fn observed_source_rows_of<I, Entity>(
        &self,
        prepared: &WorthQueryPreparedInvariantManagedIndex<Schema, I>,
        entity: &super::super::WorthQueryApplicationInvariantEntityBinding<Schema, Entity>,
    ) -> Result<
        Vec<super::WorthQueryApplicationInvariantEntity<Schema, Entity>>,
        super::super::WorthQueryInvariantAccessDenial,
    > {
        use worth_relational::facade::runtime::StructuralReadError;
        self.check_binding(&prepared.binding.binding_identity)?;
        self.check_binding(&entity.binding_identity)?;
        self.relations
            .require_entity_kind(entity.entity_kind)
            .map_err(|error| super::map_structural_error(error, "observed Source row kind"))?;
        let mut rows = Vec::new();
        for row in prepared.native.changed_rows().iter().copied() {
            let kind = match self.relations.readable_entity_kind(row) {
                Ok(kind) => kind,
                Err(StructuralReadError::RecordUnavailable) => continue,
                Err(error) => {
                    return Err(super::map_structural_error(error, "observed Source row"))
                }
            };
            self.reads.endpoint();
            if kind == Some(entity.entity_kind) {
                self.admission.entity(row, self.relations.read_budget())?;
                rows.push(self.entity(row, entity.entity_kind));
            }
        }
        Ok(rows)
    }
}

impl<S: ApplicationSchema> WorthQueryApplicationInvariantScopePlanner<'_, '_, S> {
    pub fn retain_managed_index<I: ApplicationManagedIndexBinding<S>>(
        &self,
        binding: &WorthQueryApplicationInvariantManagedIndexBinding<S, I>,
    ) -> Result<WorthQueryPreparedInvariantManagedIndex<S, I>, Denial> {
        if self.binding_identity != binding.binding_identity {
            return Err(Denial::ForeignSource);
        }
        Ok(WorthQueryPreparedInvariantManagedIndex {
            binding: binding.clone(),
            native: self.lower.prepare_managed_index_source(&binding.program)?,
        })
    }
}

pub struct WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I> {
    context: &'loan worth_relational::facade::runtime::CustomInvariantExecutionContext<'runtime>,
    prepared: &'loan WorthQueryPreparedInvariantManagedIndex<S, I>,
    match_posture: MatchPosture,
}
pub struct WorthQueryInvariantManagedIndexRead<'loan, 'runtime, S, I> {
    pair: WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I>,
    posture: invariant_read::ReadPosture,
}
impl<'borrow, 'runtime, S: ApplicationSchema>
    WorthQueryApplicationInvariantContext<'borrow, 'runtime, S>
{
    pub fn managed_index<'loan, I: ApplicationManagedIndexBinding<S>>(
        &'loan self,
        prepared: &'loan WorthQueryPreparedInvariantManagedIndex<S, I>,
    ) -> Result<WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I>, Denial> {
        self.managed_index_with_posture(prepared, MatchPosture::Candidates)
    }
    /// Explicit exact-domain access; a missing executable contract refuses at read.
    pub fn refined_managed_index<'loan, I: ApplicationManagedIndexBinding<S>>(
        &'loan self,
        prepared: &'loan WorthQueryPreparedInvariantManagedIndex<S, I>,
    ) -> Result<WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I>, Denial> {
        self.managed_index_with_posture(prepared, MatchPosture::Refined)
    }
    fn managed_index_with_posture<'loan, I: ApplicationManagedIndexBinding<S>>(
        &'loan self,
        prepared: &'loan WorthQueryPreparedInvariantManagedIndex<S, I>,
        match_posture: MatchPosture,
    ) -> Result<WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I>, Denial> {
        if self.binding_identity != prepared.binding.binding_identity {
            return Err(Denial::ForeignSource);
        }
        // This validates the actual receiving observation before any caller can
        // inspect the pair; each later query independently reborrows it again.
        self.lower.read_managed_index_source(&prepared.native)?;
        Ok(WorthQueryInvariantManagedIndexPair {
            context: self.lower,
            prepared,
            match_posture,
        })
    }
}
impl<'loan, 'runtime, S, I> WorthQueryInvariantManagedIndexPair<'loan, 'runtime, S, I> {
    pub fn committed(&self) -> WorthQueryInvariantManagedIndexRead<'loan, 'runtime, S, I> {
        WorthQueryInvariantManagedIndexRead {
            pair: Self {
                context: self.context,
                prepared: self.prepared,
                match_posture: self.match_posture,
            },
            posture: invariant_read::ReadPosture::Committed,
        }
    }
    pub fn proposed(&self) -> WorthQueryInvariantManagedIndexRead<'loan, 'runtime, S, I> {
        WorthQueryInvariantManagedIndexRead {
            pair: Self {
                context: self.context,
                prepared: self.prepared,
                match_posture: self.match_posture,
            },
            posture: invariant_read::ReadPosture::Proposed,
        }
    }
}
impl<S, I> WorthQueryInvariantManagedIndexRead<'_, '_, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    ManagedSourceValue<S, I>:
        WorthQueryApplicationProjection<S, <I::SourceBinding as ApplicationQueryBinding<S>>::Query>,
{
    pub fn query(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
    ) -> Result<Vec<ManagedSourceValue<S, I>>, WorthQueryManagedIndexInvariantDenial> {
        self.pair
            .context
            .consume_managed_index_source(&self.pair.prepared.native, |source, execution| {
                invariant_read::query::<S, I>(
                    &execution,
                    &source,
                    &self.pair.prepared.binding.program,
                    self.posture,
                    self.pair.match_posture,
                    probe,
                )
            })
            .map_err(WorthQueryManagedIndexInvariantDenial::Native)?
    }
}
