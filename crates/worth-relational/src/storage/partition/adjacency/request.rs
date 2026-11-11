//! The original request pays each actual copied adjacency index path.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;

impl AdjacencySet {
    pub(crate) fn index_historical_kind_for_request(
        &mut self,
        kind: KindId,
        relation: RelationId,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        let kinds = self
            .entries_mut()
            .historical_by_kind
            .get_or_insert_with(SharedMap::new);
        if !kinds.contains_key(&kind) {
            let mut ids = RelationSet::new();
            ids.insert_for_request(relation, (), request)?;
            kinds.insert_for_request(kind, ids, request)?;
        } else {
            let ids = kinds
                .get_mut_for_request(&kind, request)?
                .expect("the same private map still contains its selected kind");
            if !ids.contains_key(&relation) {
                ids.insert_for_request(relation, (), request)?;
            }
        }
        Ok(())
    }

    pub(crate) fn checkpoint_for_request(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        relations: Vec<RelationId>,
        structural_revisions: Vec<(KindId, VersionId)>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        let mut result = Self::new(&AdjacencyPolicy {
            backend: AdjacencyBackend::CompressedFanoutAdjacency,
            small_degree_inline_capacity: 4,
        });
        for relation in relations {
            request.consult()?;
            result
                .entries_mut()
                .current
                .insert_for_request(relation, (), request)?;
        }
        for (kind, version) in structural_revisions {
            revision(
                &mut result.entries_mut().structural_revision_by_kind,
                kind,
                version,
                request,
            )?;
        }
        Ok(result)
    }
    pub(crate) fn index_current_kind_for_request(
        &mut self,
        kind: KindId,
        relation: RelationId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        insert_kind(
            &mut self.entries_mut().current_by_kind,
            kind,
            relation,
            request,
        )
    }

    pub(crate) fn insert_at_for_request(
        &mut self,
        kind: KindId,
        relation: RelationId,
        version: VersionId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        let entries = self.entries_mut();
        if !entries.current.contains_key(&relation) {
            entries.current.insert_for_request(relation, (), request)?;
        }
        insert_kind(&mut entries.current_by_kind, kind, relation, request)?;
        insert_kind(&mut entries.historical_by_kind, kind, relation, request)?;
        revision(
            &mut entries.structural_revision_by_kind,
            kind,
            version,
            request,
        )
    }
    pub(crate) fn remove_at_for_request(
        &mut self,
        kind: KindId,
        relation: &RelationId,
        version: VersionId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        let entries = self.entries_mut();
        entries.current.remove_for_request(relation, request)?;
        if let Some(kinds) = entries.current_by_kind.as_mut()
            && kinds
                .get(&kind)
                .is_some_and(|relations| relations.contains_key(relation))
        {
            let relations = kinds.get_mut_for_request(&kind, request)?.expect(
                /* Proof: The same exclusively borrowed kind map just selected this existing key. */
                "current kind remains installed",
            );
            relations.remove_for_request(relation, request)?;
        }
        revision(
            &mut entries.structural_revision_by_kind,
            kind,
            version,
            request,
        )
    }
}
fn insert_kind(
    kinds: &mut Option<SharedMap<KindId, RelationSet>>,
    kind: KindId,
    relation: RelationId,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    let kinds = kinds.get_or_insert_with(SharedMap::new);
    if !kinds.contains_key(&kind) {
        let mut relations = RelationSet::new();
        relations.insert_for_request(relation, (), request)?;
        kinds.insert_for_request(kind, relations, request)?;
    } else if !kinds
        .get(&kind)
        .is_some_and(|relations| relations.contains_key(&relation))
    {
        kinds
            .get_mut_for_request(&kind, request)?
            .expect(
                /* Proof: The same exclusively borrowed kind map just selected this existing key. */
                "kind remains installed",
            )
            .insert_for_request(relation, (), request)?;
    }
    Ok(())
}
fn revision(
    revisions: &mut Option<SharedMap<KindId, VersionId>>,
    kind: KindId,
    version: VersionId,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    let revisions = revisions.get_or_insert_with(SharedMap::new);
    let previous = revisions.get(&kind).copied();
    let next = previous.map_or(version, |previous| previous.max(version));
    if previous != Some(next) {
        revisions.insert_for_request(kind, next, request)?;
    }
    Ok(())
}
