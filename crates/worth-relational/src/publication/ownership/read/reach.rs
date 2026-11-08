//! One-hop custody incidence. Rows never expand their other members' adjacency.
use super::*;
use crate::storage::partition::AdjacencyDirection as Direction;
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
pub(super) fn read(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    storage: &CurrentPublicationStorage,
    view: &InvariantStateView<'_>,
    records: &[RecordRef],
    scopes: &[RecordRef],
    after: Option<&InvariantStateView<'_>>,
    roster: &mut CurrentPublicationRoster,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let mut scoped = std::collections::HashSet::new();
    for record in scopes {
        checkpoint(context, 1)?;
        retain::<RecordRef>(&mut roster._funding)?;
        scoped.insert(record);
    }
    let mut reach = Reach {
        execution,
        basis,
        storage,
        view,
        after,
        roster,
        context,
        entities: BTreeSet::new(),
        publications: BTreeSet::new(),
        links: BTreeSet::new(),
        presence_sources: BTreeSet::new(),
    };
    for record in records {
        checkpoint(reach.context, 1)?;
        match *record {
            RecordRef::Entity(id) => reach.seed(id, scoped.contains(record))?,
            RecordRef::Relation(id) => {
                reach.demand(id.partition_id)?;
                let row = selector::relation(view, after, id)?;
                reach.seed(row.source, scoped.contains(record))?;
                reach.seed(row.target, scoped.contains(record))?;
                reach.roster.reached_relations.insert(id);
                if scoped.contains(record) {
                    retain::<RelationId>(&mut reach.roster._funding)?;
                    reach.roster.reached_relation_scopes.insert(id);
                }
            }
        }
    }
    // The selected set contains row IDs only, never member endpoints to expand.
    let publications = std::mem::take(&mut reach.publications);
    for id in publications {
        reach.publication(id)?;
    }
    Ok(())
}

struct Reach<'a, 'request, 'scope, 'kernel, 'authority> {
    execution: &'a ExecutionRequest<'request, 'scope>,
    basis: &'a AdmittedRelationalBranchBasis,
    storage: &'a CurrentPublicationStorage,
    view: &'a InvariantStateView<'a>,
    after: Option<&'a InvariantStateView<'a>>,
    roster: &'a mut CurrentPublicationRoster,
    context: &'a mut MapKernelContext<'kernel, 'authority>,
    entities: BTreeSet<EntityId>,
    publications: BTreeSet<EntityId>,
    links: BTreeSet<RelationId>,
    presence_sources: BTreeSet<EntityId>,
}
impl Reach<'_, '_, '_, '_, '_> {
    fn demand(
        &mut self,
        partition: crate::identity::data::PartitionId,
    ) -> Result<(), TransactionCommitError> {
        self.basis
            .inner
            .root
            .demand_head_partition(self.execution, partition)
            .map_err(|e| crate::authority::commit::preparation::storage_read_denial(self.basis, e))
    }
    fn relation(
        &mut self,
        id: RelationId,
    ) -> Result<
        crate::validation::engine::state_view::VisibleRelationMetadata,
        TransactionCommitError,
    > {
        self.demand(id.partition_id)?;
        self.view
            .relation_metadata(id)
            .ok_or_else(|| damaged(&RecordRef::Relation(id)))
    }
    fn entity(&mut self, id: EntityId) -> Result<(), TransactionCommitError> {
        if !self.entities.insert(id) {
            return Ok(());
        }
        retain::<EntityId>(&mut self.roster._funding)?;
        self.demand(id.partition_id)?;
        let kind = self
            .view
            .entity_metadata(id)
            .map(|row| row.kind_id)
            .ok_or_else(|| damaged(&RecordRef::Entity(id)))?;
        entity::load(self.roster, self.view, id, kind, self.storage, self.context)
    }
    fn incidence(
        &mut self,
        id: EntityId,
        kind: KindId,
        direction: Direction,
    ) -> Result<Vec<RelationId>, TransactionCommitError> {
        self.demand(id.partition_id)?;
        let basis = self.basis;
        let root = &basis.inner.root;
        let head = root
            .get_current_partition(id.partition_id)
            .ok_or_else(|| damaged(&RecordRef::Entity(id)))?;
        let table = head.adjacency(direction);
        checkpoint(self.context, table.navigation_work_bound() as u64 + 1)?;
        let Some(set) = table.get(id.slot_index()) else {
            return Ok(Vec::new());
        };
        checkpoint(self.context, set.navigation_work_bound() as u64 + 1)?;
        let mut found = Vec::new();
        for link in set.kind_ids(kind).iter() {
            let before = if kind == self.storage.position_link {
                super::super::observation::work(self.context)
            } else {
                None
            };
            checkpoint(self.context, 1)?;
            if let Some(before) = before {
                self.roster.observed_position_work += self.context.charged_work() - before;
            }
            retain::<RelationId>(&mut self.roster._funding)?;
            let row = self.relation(*link)?;
            if row.kind_id != kind
                || match direction {
                    Direction::Outgoing => row.source != id,
                    Direction::Incoming => row.target != id,
                }
            {
                return Err(damaged(&RecordRef::Relation(*link)));
            }
            if self.links.insert(*link) {
                let (publication, bytes) = self
                    .view
                    .relation_aspect_state(*link)
                    .and_then(|s| {
                        record::fields(s, crate::schema::current_publication_ownership::LINK)
                    })
                    .ok_or_else(|| damaged(&RecordRef::Relation(*link)))?;
                if !bytes.is_empty() {
                    return Err(payload::malformed(publication, &RecordRef::Relation(*link)));
                }
                retain::<LinkRow>(&mut self.roster._funding)?;
                self.roster.links.push(LinkRow {
                    id: *link,
                    kind,
                    publication,
                    source: row.source,
                    target: row.target,
                });
            }
            found.push(*link);
        }
        Ok(found)
    }
    fn seed(&mut self, id: EntityId, scope: bool) -> Result<(), TransactionCommitError> {
        if self.roster.reached_entities.insert(id) {
            self.claims(id)?;
        }
        if scope && self.roster.reached_scopes.insert(id) {
            self.scope(id)?;
        }
        Ok(())
    }
    fn claims(&mut self, id: EntityId) -> Result<(), TransactionCommitError> {
        retain::<EntityId>(&mut self.roster._funding)?;
        self.demand(id.partition_id)?;
        let Some(kind) = selector::entity_kind(self.view, self.after, id)? else {
            return Ok(());
        };
        if kind == self.storage.publication_kind {
            self.publications.insert(id);
        }
        for link in self.incidence(id, self.storage.member_link, Direction::Outgoing)? {
            let publication = self.relation(link)?.target;
            self.publications.insert(publication);
        }
        for link in self.incidence(id, self.storage.edge_link, Direction::Outgoing)? {
            self.entity(id)?;
            let publication = self.relation(link)?.target;
            self.publications.insert(publication);
        }
        Ok(())
    }
    fn scope(&mut self, id: EntityId) -> Result<(), TransactionCommitError> {
        if selector::entity_kind(self.view, self.after, id)?.is_none() {
            return Ok(());
        }
        // A source seed owns its own route set. Loading a catalog as a row's
        // dependency below does not perform this discovery on its other routes.
        for link in self.incidence(id, self.storage.scope_link, Direction::Incoming)? {
            let catalog = self.relation(link)?.source;
            self.entity(catalog)?;
            for link in
                self.incidence(catalog, self.storage.publication_link, Direction::Incoming)?
            {
                let publication = self.relation(link)?.source;
                self.publications.insert(publication);
            }
        }
        self.presence(id)?;
        Ok(())
    }
    fn presence(&mut self, owner: EntityId) -> Result<(), TransactionCommitError> {
        if !self.presence_sources.insert(owner) {
            return Ok(());
        }
        retain::<EntityId>(&mut self.roster._funding)?;
        for link in self.incidence(owner, self.storage.position_link, Direction::Incoming)? {
            let position = self.relation(link)?.source;
            self.entity(position)?;
        }
        Ok(())
    }
    fn publication(&mut self, id: EntityId) -> Result<(), TransactionCommitError> {
        self.entity(id)?;
        let owner = self
            .roster
            .publications
            .last()
            .filter(|r| r.entity == id)
            .map(|r| r.value.origin.meaning.source_scope)
            .ok_or_else(|| damaged(&RecordRef::Entity(id)))?;
        for link in self.incidence(id, self.storage.publication_link, Direction::Outgoing)? {
            let catalog = self.relation(link)?.target;
            self.entity(catalog)?;
            self.incidence(catalog, self.storage.scope_link, Direction::Outgoing)?;
        }
        self.presence(owner)?;
        Ok(())
    }
}

/// Replacing a publication must remove its own prior claim links. This visits
/// exactly that row's adjacency; it never expands any member or nested scope.
pub(in crate::publication::ownership) fn claims(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
    publication: CommitId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let mut roster = super::empty(execution, basis)?;
    let storage = basis
        .inner
        .root
        .schema_authority()
        .registry()
        .current_publication_storage
        .as_ref()
        .ok_or_else(|| crate::mvcc::origin_denial("publication claims require current storage"))?;
    let view = InvariantStateView::new(
        basis.inner.root.as_ref(),
        crate::identity::data::VersionId(basis.inner.root.axes().map_or(0, |a| a.storage_version)),
    );
    let mut reach = Reach {
        execution,
        basis,
        storage,
        view: &view,
        after: None,
        roster: &mut roster,
        context,
        entities: BTreeSet::new(),
        publications: BTreeSet::new(),
        links: BTreeSet::new(),
        presence_sources: BTreeSet::new(),
    };
    reach.incidence(id, storage.publication_link, Direction::Outgoing)?;
    reach.incidence(id, storage.member_link, Direction::Incoming)?;
    reach.incidence(id, storage.edge_link, Direction::Incoming)?;
    let witnesses = reach
        .roster
        .links
        .iter()
        .filter(|link| link.kind == storage.edge_link)
        .map(|link| link.source)
        .collect::<Vec<_>>();
    for witness in witnesses {
        reach.entity(witness)?;
    }
    for link in &reach.roster.links {
        checkpoint(reach.context, 1)?;
        if link.publication != publication {
            return Err(corrupt(publication, RecordRef::Relation(link.id)));
        }
    }
    Ok(roster)
}
