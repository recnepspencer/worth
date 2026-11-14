//! Restore surviving writers from the authenticated original coupled event.
use crate::execution::request_stage::checkpoint;
use crate::history::data::{CanonicalCoupledAuthoredSources, CodecRetirementClosureStop as Stop};
use crate::{
    identity::data::EntityId,
    mvcc::{
        RelationalAuthoredCoupledOwners, RelationalAuthoredRestoration,
        RelationalInstalledApplicationPublicationAuthority, RequiredOutputRolePosture,
    },
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Explicit envelope conversion retains the genuine restoration plan.
    /// With no later stages the established independent/coupled choice stays.
    pub fn readmit_authored_restoration_stage_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        restoration: RelationalAuthoredRestoration,
        later: &[EntityId],
    ) -> Result<Option<RelationalAuthoredCoupledOwners>, TransactionCommitError> {
        if later.is_empty() {
            return self.readmit_authored_restoration_coupled_owners(
                execution,
                installed,
                restoration,
            );
        }
        let owners = if restoration
            .retirement
            .coupled_authored_publication
            .is_some()
        {
            self.readmit_authored_restoration_coupled_owners(execution, installed, restoration)?
                .ok_or_else(|| {
                    crate::mvcc::origin_denial("recorded restoration lost its coupled owners")
                })?
        } else {
            if installed.runtime_instance_id != self.runtime_instance_id()
                || restoration.before.descriptor().runtime_instance_id()
                    != self.runtime_instance_id()
                || !restoration.before.is_current()
            {
                return Err(crate::mvcc::origin_denial(
                    "foreign restoration envelope basis",
                ));
            }
            let source = restoration
                .retirement
                .authored_source_retirement()
                .ok_or_else(|| {
                    crate::mvcc::origin_denial("restoration has no actual retirement")
                })?;
            let anchor = source.anchor;
            let before = restoration.before.clone();
            let retirements = restoration.publications.clone();
            RelationalAuthoredCoupledOwners {
                before,
                source: crate::mvcc::CoupledSource::Restoration {
                    anchor,
                    plan: Arc::new(restoration),
                },
                retirements,
                owners: Vec::new(),
                current: Vec::new(),
                deferred_edges: Default::default(),
                stages: Vec::new(),
                corrective_root: None,
            }
        };
        self.readmit_later_authored_stage_owners(execution, installed, owners, later)
            .map(Some)
    }

    pub fn readmit_authored_restoration_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        restoration: RelationalAuthoredRestoration,
    ) -> Result<Option<RelationalAuthoredCoupledOwners>, TransactionCommitError> {
        let Some(group) = &restoration.retirement.coupled_authored_publication else {
            return Ok(None);
        };
        let CanonicalCoupledAuthoredSources::Retirement(source) = &group.sources else {
            let recorded = match &group.sources {
                CanonicalCoupledAuthoredSources::Retirement(source) => source.anchor,
                CanonicalCoupledAuthoredSources::Edit(source) => *source,
                // Header validation admits only lists of two or more sources.
                CanonicalCoupledAuthoredSources::Stages(sources) => match sources.first() {
                    Some(source) => *source,
                    None => {
                        return Err(crate::mvcc::origin_denial(
                            "coupled authored stage list is empty",
                        ));
                    }
                },
                CanonicalCoupledAuthoredSources::Restoration(source) => source.original_anchor,
            };
            return Err(refuse(
                Cause::InvalidAuthoredAnchor,
                restoration
                    .entities
                    .first()
                    .map(|(id, _)| *id)
                    .unwrap_or(recorded),
            ));
        };
        let (owners, current) = execution
            .run_native_with_checkpoint(|context| {
                let current = crate::history::data::source_scope_closure::current(
                    execution,
                    &restoration.before,
                    Some(installed),
                    &group
                        .participants
                        .iter()
                        .map(|p| RecordRef::Entity(p.origin.meaning.source_scope))
                        .collect::<Vec<_>>(),
                    context,
                )?;
                let mut owners = Vec::with_capacity(group.participants.len());
                for part in &group.participants {
                    let required = &part.origin;
                    for item in &current {
                        checkpoint(
                            context,
                            (required.meaning.family.len() + item.origin.meaning.family.len() + 65)
                                as u64,
                        )
                        .map_err(|cause| {
                            super::authored_cascade::stopped(Stop::Request(cause.into()))
                        })?;
                    }
                    let item = current
                        .iter()
                        .find(|item| {
                            item.origin.meaning.same_route(&required.meaning)
                                && item
                                    .origin
                                    .members
                                    .iter()
                                    .any(|m| m.posture != RequiredOutputRolePosture::Retire)
                        })
                        .ok_or_else(|| {
                            refuse(
                                Cause::PublicationOwnerUnavailable {
                                    publication: required.commit.commit_id,
                                    owner: required.meaning.source_scope,
                                },
                                required.meaning.source_scope,
                            )
                        })?;
                    owners.push(self.readmit_canonical_publication(
                        execution,
                        installed,
                        &restoration.before,
                        &item.origin,
                    )?);
                }
                Ok::<_, TransactionCommitError>((owners, current))
            })
            .map_err(|cause| super::authored_cascade::stopped(Stop::Request(cause.into())))?
            .0?;
        let deferred_edges = owners
            .iter()
            .flat_map(|owner| owner.origin.relations.iter().map(|edge| edge.id))
            .collect();
        Ok(Some(RelationalAuthoredCoupledOwners {
            before: restoration.before.clone(),
            source: crate::mvcc::CoupledSource::Restoration {
                anchor: source.anchor,
                plan: Arc::new(restoration.clone()),
            },
            retirements: restoration.publications.clone(),
            owners,
            current,
            deferred_edges,
            stages: Vec::new(),
            corrective_root: None,
        }))
    }
}

fn refuse(cause: Cause, subject: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(subject),
    ))
}
