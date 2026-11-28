//! Admit the actual predecessor publications before assembling their inverse.
use worth_relational_publication::internal_port::publication::ownership::authored_restoration::RelationalAuthoredRestorationOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthorityFieldsPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::coupled_authored::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::failure;
use super::inverse::{InverseRestoration, RestorationSelection};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::codec_retirement_closure::{
    self as closure, authored_cascade::reject, charge,
};
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::mvcc::{
    RelationalAuthoredRestoration, RelationalInstalledApplicationPublicationAuthority,
};
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CreatedEntityRef, TransactionCommitError};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;

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
        if installed.runtime_instance_id() != self.runtime_instance_id() || !basis.is_current() {
            return Err(reject(Cause::InvalidAuthoredAnchor, original));
        }
        let history = closure::history_request::RuntimeHistoryRequest::new(self, execution);
        // All publication readmissions and inverse reconstruction draw from the same request.
        execution
            .in_scope(|_| {
                let selected = execution.in_scope(|_| {
                    execution.run_native_with_checkpoint(|context| {
                        charge(context, self.history.catalog_len().saturating_add(1))
                            .map_err(failure)?;
                        let envelope = history
                            .commit(retirement)
                            .map_err(failure)?
                            .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
                        if envelope.authored_source_retirement().is_some() {
                            closure::source_scope_closure::validate_historical_prefix(
                                execution, &history, basis, &envelope, original, context,
                            )?;
                        }
                        Ok::<_, TransactionCommitError>(envelope)
                    })
                });
                let envelope = selected
                    .map_err(|cause| failure(closure::request(cause)))?
                    .map_err(|cause| failure(closure::request(cause)))?
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
                        self.readmit_native_output_transition_successor(
                            execution,
                            installed,
                            basis,
                            &origin.meaning,
                        )?
                        .ok_or_else(|| reject(Cause::IncompleteRestoration, original))?,
                    );
                }
                let result = execution.in_scope(|lease| {
                    execution.run_native_with_checkpoint(|context| {
                        super::inverse::prepare(
                            RestorationSelection {
                                basis,
                                envelope: &envelope,
                                event,
                                original,
                                anchor: &anchor,
                                co_anchors,
                                publications: &publications,
                            },
                            &history,
                            context,
                            lease,
                        )
                    })
                });
                execution
                    .consult()
                    .map_err(|cause| failure(closure::request(cause)))?;
                let InverseRestoration {
                    entities,
                    relations,
                    inverse,
                    content,
                    aggregate_memberships,
                } = result
                    .map_err(|cause| failure(closure::request(cause)))?
                    .map_err(|cause| failure(closure::request(cause)))?
                    .0?;
                Ok(Some(RelationalAuthoredRestoration::from_inverse(
                    basis.clone(),
                    envelope,
                    anchor,
                    entities,
                    relations,
                    publications,
                    inverse,
                    content,
                    aggregate_memberships,
                )))
            })
            .map_err(|cause| failure(closure::request(cause)))?
    }
}
