//! Closed preflight of an external native-only verb against actual current custody.
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
    transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    /// Remove both current fact families from the real selected read image. The
    /// independently retained warm position image must still refuse the loss.
    #[doc(hidden)]
    pub fn probe_coherent_ownership_loss_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(), TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                let records = [RecordRef::Entity(owner)];
                let mut current = super::read(execution, basis, &records, context)?;
                assert_eq!(
                    current.publications.len(),
                    1,
                    "closed court selects one real publication"
                );
                current.publications.clear();
                current.codec_retired.clear();
                current.catalogs.clear();
                current.edges.clear();
                current.links.clear();
                current.positions.clear();
                super::read::anchors::validate(
                    &mut current,
                    &basis.inner.root.output_publication_positions,
                    &records,
                    context,
                )
            })
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?
            .0
    }

    /// A warm read image with independent presence but no publication row.
    #[doc(hidden)]
    pub fn probe_unmatched_ownership_presence_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(), TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                let records = [RecordRef::Entity(owner)];
                let mut current = super::read(execution, basis, &records, context)?;
                assert_eq!(
                    current.publications.len(),
                    1,
                    "real single-publication setup"
                );
                current.publications.clear();
                current.codec_retired.clear();
                super::read::anchors::validate(
                    &mut current,
                    &basis.inner.root.output_publication_positions,
                    &records,
                    context,
                )
            })
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?
            .0
    }

    /// Inspect the actual current aggregate memberships of this Query publication.
    /// Returned identifiers grant no mutation or publication standing.
    #[doc(hidden)]
    pub fn current_owned_aggregate_memberships_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<Vec<crate::identity::data::RelationId>, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
        let registry = basis.inner.root.schema_authority().registry();
        Ok(current
            .origins()
            .filter(|o| o.meaning.source_scope == owner)
            .flat_map(|o| &o.relations)
            .filter(|edge| {
                registry
                    .relation_kinds
                    .get(&edge.kind)
                    .is_some_and(|kind| kind.kind_name.starts_with("worth-aggregate-membership-"))
            })
            .map(|edge| edge.id)
            .collect())
    }

    /// Select an actual owned business edge; callers supply no publication,
    /// relation roster or ownership proof. Preparation never publishes an effect.
    #[doc(hidden)]
    pub fn prepare_owned_edge_retarget_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        target: EntityId,
    ) -> Result<(), TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
        let registry = basis.inner.root.schema_authority().registry();
        let edge = current
            .origins()
            .filter(|origin| origin.meaning.source_scope == owner)
            .flat_map(|origin| &origin.relations)
            .find(|edge| !registry.native_private_kinds.contains(&edge.kind))
            .ok_or_else(|| {
                crate::mvcc::origin_denial(
                    "owned edge retarget court requires a real business edge",
                )
            })?;
        let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id: edge.id,
                kind_id: edge.kind,
                source: EntityReference::Existing(edge.source),
                target: EntityReference::Existing(target),
            },
        ));
        let mut transaction = self
            .begin_branch_transaction(basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| {
                crate::mvcc::origin_denial(format!("actual endpoint probe admission: {cause:?}"))
            })?;
        transaction
            .push_batch(
                WorkerIntentBatch::new("external-owned-edge-retarget").push(intent),
                execution.allocation_policy(),
            )
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        let _unpublished = self.prepare_branch_transaction(execution, transaction)?;
        Ok(())
    }
}

impl RelationalRuntime {
    /// Descriptive publication sources reached for one real Query output record.
    #[doc(hidden)]
    pub fn current_ownership_sources_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: EntityId,
    ) -> Result<Vec<EntityId>, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(record)])?;
        Ok(current
            .origins()
            .map(|origin| origin.meaning.source_scope)
            .collect())
    }
}

/// Actual private rows decoded by a one-hop custody read, with their source loci.
#[derive(Debug)]
pub struct CurrentOwnershipContactsForTest {
    pub publication_rows: Vec<(EntityId, EntityId)>,
    pub presence_rows: Vec<(EntityId, EntityId)>,
    /// Each incidence is charged before its current metadata is read.
    pub scope_links: usize,
    pub position_links: usize,
    /// Actual meter deltas for the per-position incidence checkpoints.
    pub position_work: u64,
}
impl RelationalRuntime {
    #[doc(hidden)]
    pub fn current_ownership_contacts_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: EntityId,
    ) -> Result<CurrentOwnershipContactsForTest, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(record)])?;
        Ok(CurrentOwnershipContactsForTest::from_current(
            &current, basis,
        ))
    }
}

impl RelationalRuntime {
    /// Exercise both existing cold route doors using a real Query publication's
    /// canonical route. Only descriptive identifiers enter this readonly probe.
    #[doc(hidden)]
    pub fn probe_current_history_contradiction_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        publication: crate::history::data::CommitId,
        source: EntityId,
    ) -> Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError> {
        let envelope = self
            .history
            .canonical_envelope(publication)
            .ok_or_else(|| {
                crate::mvcc::origin_denial("real Query publication history unavailable")
            })?;
        let meaning = envelope
            .output_origins()
            .find(|o| o.meaning.source_scope == source && o.source_scope_owned)
            .map(|o| o.meaning.clone())
            .ok_or_else(|| crate::mvcc::origin_denial("actual owned Query route missing"))?;
        let absence = self
            .readmit_required_output_route_absence(execution, installed, basis, &meaning)
            .map(|_| ());
        let retirement = self
            .readmit_native_output_transition_successor(execution, installed, basis, &meaning)
            .map(|_| ());
        Ok(vec![absence, retirement])
    }
}

impl RelationalRuntime {
    /// The actual cascade owner, distinct from ordinary one-hop custody reach.
    #[doc(hidden)]
    pub fn current_ownership_descendants_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        source: EntityId,
    ) -> Result<Vec<EntityId>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                let latest = crate::history::data::source_scope_closure::current(
                    execution,
                    basis,
                    None,
                    &[RecordRef::Entity(source)],
                    context,
                )?;
                let closure =
                    crate::history::data::source_scope_closure::select(&[source], &latest, context)
                        .expect("actual current cascade closure under the court request");
                Ok(closure
                    .into_iter()
                    .map(|p| p.origin.meaning.source_scope)
                    .collect())
            })
            .map_err(super::request)?
            .0
    }
}

impl CurrentOwnershipContactsForTest {
    pub(super) fn from_current(
        current: &super::CurrentPublicationRoster,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Self {
        let storage = basis
            .inner
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .as_ref();
        Self {
            position_work: current.observed_position_work,
            publication_rows: current
                .publications
                .iter()
                .chain(&current.codec_retired)
                .map(|r| (r.value.origin.meaning.source_scope, r.entity))
                .collect(),
            presence_rows: current
                .positions
                .iter()
                .map(|(id, p)| (p.meaning.source_scope, *id))
                .collect(),
            scope_links: storage.map_or(0, |s| {
                current
                    .links
                    .iter()
                    .filter(|l| l.kind == s.scope_link)
                    .count()
            }),
            position_links: storage.map_or(0, |s| {
                current
                    .links
                    .iter()
                    .filter(|l| l.kind == s.position_link)
                    .count()
            }),
        }
    }
}
impl RelationalRuntime {
    /// Read-only observation of actual one-hop reads during public Query edits.
    #[doc(hidden)]
    pub fn record_current_ownership_contacts_for_test<
        F: Fn(CurrentOwnershipContactsForTest) + 'static,
    >(
        &self,
        observer: F,
    ) -> impl Sized + use<F> {
        super::observation::install(Box::new(observer))
    }
}
