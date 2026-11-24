//! Restore surviving writers from the authenticated original coupled event.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_owners::RelationalAuthoredCoupledOwnersOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthorityFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::authored_restoration::RelationalAuthoredRestorationFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::custody::RelationalRequiredOutputPublicationFieldsPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::coupled_authored::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use crate::history::data::{CanonicalCoupledAuthoredSources, CodecRetirementClosureStop as Stop};
use crate::identity::data::EntityId;
use crate::mvcc::RelationalAuthoredCoupledOwners;
use crate::mvcc::RelationalAuthoredRestoration;
use crate::mvcc::RelationalInstalledApplicationPublicationAuthority;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;

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
        let owners = if (*restoration
            .retirement()
            .owner_coupled_authored_publication())
        .is_some()
        {
            self.readmit_authored_restoration_coupled_owners(execution, installed, restoration)?
                .ok_or_else(|| {
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "recorded restoration lost its coupled owners",
                    )
                })?
        } else {
            if installed.runtime_instance_id() != self.runtime_instance_id()
                || restoration.before().descriptor().runtime_instance_id()
                    != self.runtime_instance_id()
                || !restoration.before().is_current()
            {
                return Err(
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "foreign restoration envelope basis",
                    ),
                );
            }
            let source = restoration
                .retirement()
                .authored_source_retirement()
                .ok_or_else(|| {
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "restoration has no actual retirement",
                    )
                })?;
            let anchor = source.anchor;
            let before = restoration.before().clone();
            let retirements = restoration.publications().clone();
            RelationalAuthoredCoupledOwners::bind_restoration(
                before,
                crate::mvcc::CoupledSource::Restoration {
                    anchor,
                    plan: Arc::new(restoration),
                },
                retirements,
                Vec::new(),
                Vec::new(),
                Default::default(),
                Vec::new(),
                None,
            )
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
        let Some(group) = restoration
            .retirement()
            .owner_coupled_authored_publication()
        else {
            return Ok(None);
        };
        let Some(source) = restoration.retirement().authored_source_retirement() else {
            let recorded = match &group.sources {
                CanonicalCoupledAuthoredSources::Retirement { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRetirement { prefix, .. } => {
                    prefix.anchor
                }
                CanonicalCoupledAuthoredSources::Edit { sources } => {
                    *sources.first().ok_or_else(|| {
                        crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                            "coupled edit source list has no authored prefix",
                        )
                    })?
                }
                CanonicalCoupledAuthoredSources::Restoration { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRestoration { prefix, .. } => {
                    prefix.original_anchor
                }
            };
            return Err(refuse(
                Cause::InvalidAuthoredAnchor,
                restoration
                    .entities()
                    .first()
                    .map(|(id, _)| *id)
                    .unwrap_or(recorded),
            ));
        };
        let (owners, current) = execution
            .run_native_with_checkpoint(|context| {
                let current = crate::publication::ownership::source_scope::current(
                    execution,
                    restoration.before(),
                    Some(installed.source_scope_ownership()),
                    &group
                        .participants
                        .iter()
                        .map(|p| RecordRef::Entity(p.origin.meaning.source_scope))
                        .collect::<Vec<_>>(),
                    context,
                )?;
                let mut owners = Vec::with_capacity(group.participants.len());
                for part in group.participants.iter().filter(|part| part.is_required()) {
                    let required = &part.origin;
                    let origin = super::coupled_selection::require_current_origin(
                        context, required, &current,
                    )?;
                    owners.push(self.readmit_canonical_publication(
                        execution,
                        installed,
                        restoration.before(),
                        origin,
                    )?);
                }
                Ok::<_, TransactionCommitError>((owners, current))
            })
            .map_err(|cause| super::authored_cascade::stopped(Stop::Request(cause.into())))?
            .0?;
        let deferred_edges = owners
            .iter()
            .flat_map(|owner| owner.origin().relations.iter().map(|edge| edge.id))
            .collect();
        Ok(Some(RelationalAuthoredCoupledOwners::bind_restoration(
            restoration.before().clone(),
            crate::mvcc::CoupledSource::Restoration {
                anchor: source.anchor,
                plan: Arc::new(restoration.clone()),
            },
            restoration.publications().clone(),
            owners,
            current,
            deferred_edges,
            Vec::new(),
            None,
        )))
    }
}

fn refuse(cause: Cause, subject: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(subject),
    ))
}
