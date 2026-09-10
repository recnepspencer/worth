use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::scopes_overlap;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
mod work;
enum CauseCommitAuthority<'a> {
    Published,
    Prepared(&'a ProducedAspectDelta),
}

impl SignalGraph {
    pub(crate) fn validate_prepared_causes_before_evaluation(
        &self,
        consumer: NodeId,
        causes: &[ResolvedDependencyCause],
        delta: &ProducedAspectDelta,
        version: crate::data::aspect::AspectVersion,
        regions: &[crate::data::output::ChangedRegion],
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(causes.len()))?;
        for cause in causes {
            self.admit_prepared_cause_validation(consumer, cause, delta, regions.len(), work)?;
            let projected = if cause.key.producer == delta.producer {
                Some(self.node_version_after_evaluation(
                    cause.key.producer,
                    cause.key.aspect,
                    cause.key.edge_scope.as_ref(),
                    version,
                    regions,
                    work,
                )?)
            } else {
                None
            };
            self.validate_pending_cause(
                consumer,
                cause,
                CauseCommitAuthority::Prepared(delta),
                projected,
            )?;
        }
        Ok(())
    }

    pub(crate) fn validate_pending_causes(
        &self,
        consumer: NodeId,
        causes: &[ResolvedDependencyCause],
    ) -> Result<(), SignalError> {
        for cause in causes {
            self.validate_pending_cause(consumer, cause, CauseCommitAuthority::Published, None)?;
        }
        Ok(())
    }

    pub(crate) fn validate_prepared_pending_causes(
        &self,
        consumer: NodeId,
        causes: &[ResolvedDependencyCause],
        delta: &ProducedAspectDelta,
    ) -> Result<(), SignalError> {
        for cause in causes {
            self.validate_pending_cause(
                consumer,
                cause,
                CauseCommitAuthority::Prepared(delta),
                None,
            )?;
        }
        Ok(())
    }

    fn validate_pending_cause(
        &self,
        consumer: NodeId,
        cause: &ResolvedDependencyCause,
        commit_authority: CauseCommitAuthority<'_>,
        projected_version: Option<u64>,
    ) -> Result<(), SignalError> {
        self.validate_cause_identity_axes(consumer, cause)?;
        let edge = self
            .current_runtime_dependencies_of(consumer)?
            .iter()
            .find(|edge| {
                edge.source() == cause.key.producer
                    && edge.aspect() == cause.key.aspect
                    && edge.scope_ref() == cause.key.edge_scope.as_ref()
            })
            .ok_or_else(|| {
                SignalError::invalid_input(
                    "pending dependency cause does not match a current dependency edge",
                )
            })?;
        let snapshot = self
            .get_dep_snapshot(consumer)?
            .entries()
            .iter()
            .find(|entry| {
                entry.source == cause.key.producer
                    && entry.aspect == cause.key.aspect
                    && entry.scope.as_ref() == edge.scope_ref()
            })
            .ok_or_else(|| {
                SignalError::invalid_input(
                    "pending dependency cause has no matching dependency snapshot",
                )
            })?;
        if snapshot.cached_version != cause.binding_axes.cached_version {
            return Err(SignalError::invalid_input(
                "pending dependency cause cached version drifted from its dependency snapshot",
            ));
        }
        let current_version = match projected_version {
            Some(version) => version,
            None => self.node_version_for_scope(
                cause.key.producer,
                cause.key.aspect,
                cause.key.edge_scope.as_ref(),
            )?,
        };
        if current_version != cause.binding_axes.committed_version {
            return Err(SignalError::invalid_input(
                "pending dependency cause committed version drifted from producer authority",
            ));
        }
        if !self.commit_authority_matches(cause, commit_authority) {
            return Err(SignalError::invalid_input(
                "pending dependency cause has no matching performed output commit",
            ));
        }
        let expected_scopes = edge.scope_ref().map(std::slice::from_ref).unwrap_or(&[]);
        if cause.changed_scopes.as_slice() != expected_scopes {
            return Err(SignalError::invalid_input(
                "pending dependency cause scope is not normalized to its dependency edge",
            ));
        }
        Ok(())
    }

    fn commit_authority_matches(
        &self,
        cause: &ResolvedDependencyCause,
        authority: CauseCommitAuthority<'_>,
    ) -> bool {
        let ordinal = cause.binding_axes.output_commit_ordinal;
        if ordinal.0 == 0 {
            return false;
        }
        let delta = match authority {
            CauseCommitAuthority::Published => self.cause_sets.published_output_commit(ordinal),
            CauseCommitAuthority::Prepared(delta) if delta.output_commit_ordinal == ordinal => {
                Some(delta)
            }
            CauseCommitAuthority::Prepared(_) => self.cause_sets.published_output_commit(ordinal),
        };
        delta.is_some_and(|delta| {
            delta.output_commit_ordinal == ordinal
                && delta.producer == cause.key.producer
                && delta.changes.as_slice().iter().any(|change| {
                    let scope_matches = cause.key.edge_scope.as_ref().is_none_or(|edge_scope| {
                        change.changed_scopes.is_empty()
                            || change
                                .changed_scopes
                                .iter()
                                .any(|changed| scopes_overlap(changed, edge_scope))
                    });
                    change.aspect == cause.key.aspect
                        && change.committed_version == cause.binding_axes.committed_version
                        && scope_matches
                })
        })
    }

    fn validate_cause_identity_axes(
        &self,
        consumer: NodeId,
        cause: &ResolvedDependencyCause,
    ) -> Result<(), SignalError> {
        let axes = &cause.binding_axes;
        let key = &cause.key;
        let key_matches_binding = key.graph_instance == axes.graph_instance
            && key.consumer == axes.consumer
            && key.dependency_revision == axes.dependency_revision
            && key.producer == axes.producer
            && key.aspect == axes.aspect
            && key.edge_scope == axes.edge_scope;
        let binding_matches_graph = axes.graph_instance == self.runtime_instance_id()
            && axes.consumer == consumer
            && axes.dependency_revision == self.dependency_revision(consumer)?
            && self.is_alive(axes.producer);
        if !key_matches_binding || !binding_matches_graph {
            return Err(SignalError::invalid_input(
                "pending dependency cause binding axes do not match current graph authority",
            ));
        }
        Ok(())
    }
}
