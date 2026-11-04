//! Restore plans reconstruct exactly recorded members and owned edges, with fresh generations.
use super::{authored_cascade::reject, charge, CodecRetirementClosureStop as Stop};
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::CommitId, identity::data::EntityId,
    mvcc::*, publication::data::AuthoredRetirementRefusal as Cause, runtime::RelationalRuntime,
    storage::overlay::PartitionAccess, symbols::data::ClientKey, transactions::data::*,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::facade::*;
mod creation_patch;
pub(super) use creation_patch::creation_patch;

fn failure(stop: Stop) -> TransactionCommitError {
    match stop {
        Stop::Payload(detail) => crate::mvcc::origin_denial(detail),
        Stop::Request(cause) => TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        }),
    }
}

impl RelationalRuntime {
    pub fn prepare_authored_source_restoration(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        retirement: CommitId,
        original: EntityId,
        anchor: CreatedEntityRef,
    ) -> Result<Option<RelationalAuthoredRestoration>, TransactionCommitError> {
        self.prepare_authored_source_restoration_with_co_anchors(
            execution,
            installed,
            basis,
            retirement,
            (original, anchor),
            &[],
        )
    }

    /// Every co-anchor is recreated by its own authored inverse Create.
    pub fn prepare_authored_source_restoration_with_co_anchors(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        retirement: CommitId,
        anchor: (EntityId, CreatedEntityRef),
        co_anchors: &[(EntityId, CreatedEntityRef)],
    ) -> Result<Option<RelationalAuthoredRestoration>, TransactionCommitError> {
        let (original, anchor) = anchor;
        if installed.runtime_instance_id != self.runtime_instance_id() || !basis.is_current() {
            return Err(reject(Cause::InvalidAuthoredAnchor, original));
        }
        // All publication readmissions and inverse reconstruction draw from the same request.
        execution
            .in_scope(|_| {
                let selected = execution.in_scope(|lease| {
                    ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                        lease,
                        None,
                        |context| {
                            charge(context, self.history.catalog_len().saturating_add(1))
                                .map_err(failure)?;
                            let envelope = self
                                .history
                                .canonical_envelope(retirement)
                                .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
                            if envelope.authored_source_retirement().is_some() {
                                super::source_scope_closure::validate_historical_prefix(
                                    execution, self, basis, &envelope, original, context,
                                )?;
                            }
                            Ok::<_, TransactionCommitError>(envelope)
                        },
                    )
                });
                let envelope = selected
                    .map_err(|cause| failure(super::request(cause)))?
                    .map_err(|cause| failure(super::request(cause)))?
                    .0?;
                let Some(event) = envelope.authored_source_retirement() else {
                    return Ok(None);
                };
                if event.anchor != original {
                    return Err(reject(Cause::InvalidAuthoredAnchor, original));
                }
                let mut publications = Vec::with_capacity(event.publications.len());
                for origin in &event.publications {
                    publications.push(
                        self.readmit_authored_output_retirement_successor(
                            execution,
                            installed,
                            basis,
                            &origin.meaning,
                        )?
                        .ok_or_else(|| reject(Cause::IncompleteRestoration, original))?,
                    );
                }
                let result = execution.in_scope(|lease| {
                    ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                        lease,
                        None,
                        |context| {
                            charge(
                                context,
                                co_anchors.len().saturating_mul(event.co_anchors.len() + 1),
                            )
                            .map_err(failure)?;
                            if co_anchors.len() != event.co_anchors.len() {
                                return Err(reject(Cause::IncompleteRestoration, original));
                            }
                            for (index, (id, _)) in co_anchors.iter().enumerate() {
                                charge(context, index + 1).map_err(failure)?;
                                if !event.co_anchors.iter().any(|(old, _)| old == id)
                                    || co_anchors[..index].iter().any(|(old, _)| old == id)
                                {
                                    return Err(reject(Cause::IncompleteRestoration, *id));
                                }
                            }
                            for (original, anchor) in std::iter::once((original, &anchor))
                                .chain(co_anchors.iter().map(|(id, target)| (*id, target)))
                            {
                                basis
                                    .inner
                                    .root
                                    .demand_full_partition(execution, original.partition_id)
                                    .map_err(|cause| failure(super::request(cause)))?;
                                let partition = basis
                                    .inner
                                    .root
                                    .partition_state(original.partition_id)
                                    .map_err(|cause| failure(cause.into()))?
                                    .ok_or_else(|| {
                                        reject(Cause::InvalidAuthoredAnchor, original)
                                    })?;
                                let row = partition
                                    .entity_arena
                                    .get(&original)
                                    .filter(|row| {
                                        row.retired_at() == Some(envelope.commit.version_id)
                                            && row.kind_id() == Some(anchor.kind_id)
                                    })
                                    .ok_or_else(|| {
                                        reject(Cause::InvalidAuthoredAnchor, original)
                                    })?;
                                let _ = row;
                            }
                            // Fund the fixed-width multi-publication proof comparisons before returning custody.
                            charge(
                                context,
                                publications
                                    .len()
                                    .saturating_mul(publications.len().saturating_add(1)),
                            )
                            .map_err(failure)?;
                            let mut entities = vec![(original, anchor.clone())];
                            entities.extend_from_slice(co_anchors);
                            for prior in &publications {
                                for member in &prior.origin.members {
                                    charge(context, 1).map_err(failure)?;
                                    entities.push((
                                        member.id,
                                        CreatedEntityRef {
                                            partition_id: member.id.partition_id,
                                            kind_id: member.kind,
                                            client_key: ClientKey::raw(format!(
                                                "authored-restore:{}:{}:{}",
                                                member.id.partition_id.0,
                                                member.id.local_slot_value(),
                                                member.id.generation_value()
                                            )),
                                        },
                                    ));
                                }
                            }
                            let mut inverse = Vec::new();
                            let mut content =
                                crate::content::RelationalContentAttachments::default();
                            let mut relations = Vec::new();
                            let mut aggregate_memberships = Vec::new();
                            for prior in &publications {
                                let predecessor =
                                    prior.origin.publication.predecessor().ok_or_else(|| {
                                        reject(Cause::CorruptPublication, original)
                                    })?;
                                let live = self
                                    .history
                                    .canonical_envelope(predecessor)
                                    .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
                                for member in &prior.origin.members {
                                    let plan = basis
                                        .inner
                                        .root
                                        .schema_authority()
                                        .entity_aspect_plan(member.kind)
                                        .ok_or_else(|| {
                                            reject(Cause::CorruptPublication, member.id)
                                        })?;
                                    let payload = super::membership::reconstruct(
                                        self,
                                        &live,
                                        member.id,
                                        plan,
                                        context,
                                        lease,
                                        basis.inner.root.schema_authority(),
                                    )
                                    .map_err(failure)?;
                                    live.content
                                        .retain_restoration_content(
                                            execution,
                                            payload.as_ref().and_then(|p| p.state.as_ref()),
                                            &mut content,
                                        )
                                        .map_err(crate::content::commit_denial)?;
                                    charge(context, entities.len()).map_err(failure)?;
                                    let target = entities
                                        .iter()
                                        .find(|(old, _)| *old == member.id)
                                        .expect("complete original roster")
                                        .1
                                        .clone();
                                    let aspect_patch = creation_patch(
                                        payload.as_ref().and_then(|p| p.state.as_ref()),
                                        plan,
                                        context,
                                    )
                                    .map_err(failure)?;
                                    inverse.push(MutationIntent::Create(
                                        CreateIntent::EntityAspects(EntityAspectCreateIntent {
                                            partition_id: target.partition_id,
                                            kind_id: target.kind_id,
                                            client_key: target.client_key,
                                            aspect_patch,
                                        }),
                                    ));
                                }
                                let live_origin = live
                                    .output_origin_for_route(&prior.origin.meaning, context)
                                    .map_err(failure)?;
                                for edge in &live_origin.relations {
                                    let plan = basis
                                        .inner
                                        .root
                                        .schema_authority()
                                        .relation_aspect_plan(edge.kind)
                                        .ok_or_else(|| {
                                            reject(Cause::CorruptPublication, original)
                                        })?;
                                    let payload = super::relation_payload::reconstruct(
                                        self,
                                        &live,
                                        edge.id,
                                        plan,
                                        context,
                                        lease,
                                        basis.inner.root.schema_authority(),
                                    )
                                    .map_err(failure)?;
                                    live.content
                                        .retain_restoration_content(
                                            execution,
                                            payload.as_ref().and_then(|p| p.state.as_ref()),
                                            &mut content,
                                        )
                                        .map_err(crate::content::commit_denial)?;
                                    if basis
                                        .inner
                                        .root
                                        .schema_authority()
                                        .registry()
                                        .native_private_kinds
                                        .contains(&edge.kind)
                                    {
                                        let (state, funding) = match payload {
                                            Some(payload) => {
                                                let (state, funding) =
                                                    payload.into_retained_parts();
                                                (state, Some(funding))
                                            }
                                            None => (None, None),
                                        };
                                        aggregate_memberships.push(
                                            crate::aggregate_identity::RestoredMembership::capture(
                                                edge.id,
                                                edge.kind,
                                                edge.source,
                                                state,
                                                funding,
                                            ),
                                        );
                                        continue;
                                    }
                                    charge(context, entities.len().saturating_mul(2))
                                        .map_err(failure)?;
                                    let resolve = |id| {
                                        entities
                                            .iter()
                                            .find(|(old, _)| *old == id)
                                            .map_or(EntityReference::Existing(id), |(_, target)| {
                                                EntityReference::Created(target.clone())
                                            })
                                    };
                                    let target = CreatedRelationRef {
                                        partition_id: edge.id.partition_id,
                                        kind_id: edge.kind,
                                        client_key: ClientKey::raw(format!(
                                            "authored-restore-edge:{}:{}:{}",
                                            edge.id.partition_id.0,
                                            edge.id.local_slot_value(),
                                            edge.id.generation_value()
                                        )),
                                        source: resolve(edge.source),
                                        target: resolve(edge.target),
                                    };
                                    inverse.push(MutationIntent::Create(
                                        CreateIntent::RelationAspects(RelationAspectCreateIntent {
                                            partition_id: target.partition_id,
                                            kind_id: target.kind_id,
                                            client_key: target.client_key.clone(),
                                            source: target.source.clone(),
                                            target: target.target.clone(),
                                            aspect_patch: creation_patch(
                                                payload.as_ref().and_then(|p| p.state.as_ref()),
                                                plan,
                                                context,
                                            )
                                            .map_err(failure)?,
                                        }),
                                    ));
                                    relations.push((edge.id, target));
                                }
                            }
                            Ok::<_, TransactionCommitError>((
                                entities,
                                relations,
                                inverse,
                                content,
                                aggregate_memberships,
                            ))
                        },
                    )
                });
                execution
                    .consult()
                    .map_err(|cause| failure(super::request(cause)))?;
                let (entities, relations, inverse, content, aggregate_memberships) = result
                    .map_err(|cause| failure(super::request(cause)))?
                    .map_err(|cause| failure(super::request(cause)))?
                    .0?;
                Ok(Some(RelationalAuthoredRestoration {
                    before: basis.clone(),
                    retirement: envelope,
                    anchor,
                    entities,
                    relations,
                    publications,
                    inverse,
                    content,
                    aggregate_memberships,
                }))
            })
            .map_err(|cause| failure(super::request(cause)))?
    }
}
