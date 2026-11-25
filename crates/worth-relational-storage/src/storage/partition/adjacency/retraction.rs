//! Invert one adjacency delta of a superseded coupled round (#119).
use super::request::insert_kind;
use super::*;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;

impl AdjacencySet {
    /// Undo one delta on this anchor. A created relation leaves `current`,
    /// its current kind bucket and its historical kind bucket; a deleted one
    /// returns to `current` and its current kind bucket (its historical entry
    /// was never removed). `revision`, when given, is the kind's structural
    /// revision to restore (`None` removes it): the image before the chain's
    /// head, used only when no other live member touched this anchor and kind.
    /// O(log degree).
    pub fn retract_delta_for_request(
        &mut self,
        kind: KindId,
        relation: RelationId,
        created: bool,
        revision: Option<Option<VersionId>>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        request.consult()?;
        let entries = self.entries_mut();
        if created {
            entries.current.remove_for_request(&relation, request)?;
            // Historical buckets only ever grow, so an emptied one did not
            // exist before the chain; current buckets keep their shape.
            for (kinds, historical) in [
                (&mut entries.current_by_kind, false),
                (&mut entries.historical_by_kind, true),
            ] {
                if let Some(kinds) = kinds.as_mut()
                    && kinds
                        .get(&kind)
                        .is_some_and(|relations| relations.contains_key(&relation))
                {
                    let relations = kinds.get_mut_for_request(&kind, request)?.expect(
                        /* Proof: The same exclusively borrowed kind map just selected this existing key. */
                        "kind remains installed",
                    );
                    relations.remove_for_request(&relation, request)?;
                    if historical && relations.is_empty() {
                        kinds.remove_for_request(&kind, request)?;
                    }
                }
            }
        } else {
            if !entries.current.contains_key(&relation) {
                entries.current.insert_for_request(relation, (), request)?;
            }
            insert_kind(&mut entries.current_by_kind, kind, relation, request)?;
        }
        if let Some(revision) = revision {
            let revisions = entries
                .structural_revision_by_kind
                .get_or_insert_with(SharedMap::new);
            match revision {
                Some(version) => {
                    revisions.insert_for_request(kind, version, request)?;
                }
                None => {
                    revisions.remove_for_request(&kind, request)?;
                }
            }
        }
        Ok(())
    }
}
