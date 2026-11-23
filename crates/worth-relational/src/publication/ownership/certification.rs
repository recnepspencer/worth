//! Closed preflight of an external native-only verb against actual current custody.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::transactions::data::*;
use worth_execution::ExecutionRequest;
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

pub(crate) fn probe_coherent_ownership_loss(
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

pub(crate) fn probe_unmatched_ownership_presence(
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

pub(crate) fn current_owned_aggregate_memberships(
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

pub(crate) fn current_ownership_sources(
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

pub(crate) fn current_ownership_contacts(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    record: EntityId,
) -> Result<CurrentOwnershipContactsForTest, TransactionCommitError> {
    let current = super::select_current(execution, basis, &[RecordRef::Entity(record)])?;
    Ok(CurrentOwnershipContactsForTest::from_current(
        &current, basis,
    ))
}

pub(crate) fn current_ownership_descendants(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    source: EntityId,
) -> Result<Vec<EntityId>, TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            let latest = crate::publication::ownership::source_scope::current(
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

pub(crate) fn record_current_ownership_contacts<
    F: Fn(CurrentOwnershipContactsForTest) + 'static,
>(
    observer: F,
) -> impl Sized + use<F> {
    super::observation::install(Box::new(observer))
}

pub(crate) fn owned_edge_retarget_intent(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
    target: EntityId,
) -> Result<MutationIntent, TransactionCommitError> {
    let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
    let registry = basis.inner.root.schema_authority().registry();
    let edge = current
        .origins()
        .filter(|origin| origin.meaning.source_scope == owner)
        .flat_map(|origin| &origin.relations)
        .find(|edge| !registry.native_private_kinds.contains(&edge.kind))
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "owned edge retarget court requires a real business edge",
            )
        })?;
    Ok(MutationIntent::Relation(
        RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
            relation_id: edge.id,
            kind_id: edge.kind,
            source: EntityReference::Existing(edge.source),
            target: EntityReference::Existing(target),
        }),
    ))
}

pub(crate) fn source_route_meaning(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    source: EntityId,
) -> Result<crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning, TransactionCommitError>{
    envelope
        .output_origins()
        .find(|o| o.meaning.source_scope == source && o.source_scope_owned)
        .map(|o| o.meaning.clone())
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "actual owned Query route missing",
            )
        })
}
