//! After the commit: record every bound output tree at it, then adopt once.
use super::*;
use crate::application_entry::history::outputs::RecordingForest;
use crate::application_entry::mutation::envelope::WorthQueryEditEnvelopeReceipt;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_execution::{
    facade::product::WorthQueryExpectedProductBranch,
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryEnvelopeConstraint, WorthQueryDocumentHistoryRead,
        WorthQueryDocumentHistoryReservation, WorthQueryRecordingKind,
    },
};
use WorthQueryHistoryEditEnvelopeDenial as Denial;

/// The reserved record and the private candidate that holds the commit.
type Owners<'runtime, S> = (
    WorthQueryDocumentHistoryReservation,
    WorthQueryExpectedProductBranch<'runtime, S>,
);

/// What one committed envelope is sealed from.
pub(super) struct Committed<'a, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub(super) document: &'a WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub(super) principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
    pub(super) scope: &'a WorthQueryRequestScope,
    pub(super) constraint: &'a WorthQueryDocumentHistoryEnvelopeConstraint,
    pub(super) receipt: &'a WorthQueryEditEnvelopeReceipt,
}

impl<'runtime, S, H, Payload, Outputs> Committed<'_, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Make the commit the document's next step. Bound output trees are
    /// recorded first, over as many phases as their producers need, and the
    /// head moves once. Every refusal discards the candidate.
    pub(super) fn seal(
        &self,
        owners: Owners<'runtime, S>,
    ) -> Result<WorthQueryHistoryStep, Denial> {
        let access = program_publication_access();
        let owner = &self.document.owner;
        let mut owners = Some(owners);
        let mut first = true;
        loop {
            let advanced = owner
                .application()
                .with_application_advancement(self.scope, |phase| {
                    let sealed =
                        self.seal_in_phase(&phase, &mut owners, std::mem::take(&mut first));
                    if sealed.is_err()
                        && let Some((reservation, candidate)) = owners.take()
                    {
                        owner.abandon_history_envelope(
                            Some(&phase),
                            &access,
                            reservation,
                            Some(candidate),
                        );
                    }
                    sealed
                });
            match advanced {
                Ok(Ok(Some(step))) => return Ok(step),
                // A producer is still working on the candidate.
                Ok(Ok(None)) => std::thread::yield_now(),
                Ok(Err(denial)) => return Err(denial),
                Err(cause) => {
                    // The request could not advance again, so the commit stays private.
                    if let Some((reservation, candidate)) = owners.take() {
                        owner.abandon_history_envelope(None, &access, reservation, Some(candidate));
                    }
                    return Err(Denial::History(
                        WorthQueryApplicationHistoryDenial::ExecutionRequest(cause),
                    ));
                }
            }
        }
    }

    /// `Ok(None)` keeps both owners for the next phase.
    fn seal_in_phase(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        owners: &mut Option<Owners<'runtime, S>>,
        first: bool,
    ) -> Result<Option<WorthQueryHistoryStep>, Denial> {
        let access = program_publication_access();
        let owner = &self.document.owner;
        let (_, candidate) = owners
            .as_ref()
            .expect("an unsealed envelope keeps its owners");
        if first
            && let Err((stage, denial)) =
                owner.confirm_history_envelope_scopes(phase, &access, candidate, self.constraint)
        {
            return Err(match stage {
                Some(ordinal) => Denial::Stage { ordinal, denial },
                None => Denial::Adoption(denial),
            });
        }
        let branch = candidate.branch();
        super::super::recording::seal(
            &self.document.outputs,
            phase,
            branch,
            WorthQueryRecordingKind::Envelope,
            owners,
            |owners, forest| {
                let (reservation, candidate) = owners
                    .as_ref()
                    .expect("an unsealed envelope keeps its owners");
                self.record_outputs(phase, reservation, candidate, forest)
            },
            |owners, completed| {
                let (reservation, candidate) = owners
                    .take()
                    .expect("an unsealed envelope keeps its owners");
                let commit = self.receipt.receipt();
                match completed {
                    Some(completed) => owner.adopt_history_envelope_output_forest(
                        phase,
                        &access,
                        reservation,
                        candidate,
                        commit,
                        completed,
                    ),
                    None => {
                        owner.adopt_history_envelope(phase, &access, reservation, candidate, commit)
                    }
                }
                .map_err(|failure| {
                    let (reservation, candidate, denial) = failure.into_parts();
                    *owners = Some((reservation, candidate));
                    Denial::Adoption(denial)
                })
            },
            |_| Ok(()),
        )
        .map_err(Denial::Outputs)?
    }

    /// Advance every bound tree on the candidate under the envelope's own
    /// admissions. True once all of them are recorded at one endpoint.
    fn record_outputs(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reservation: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        forest: &mut RecordingForest<S>,
    ) -> Result<bool, Denial> {
        let access = program_publication_access();
        let owner = &self.document.owner;
        let runtime = owner.application();
        if forest.needs_origin() {
            let origin = owner
                .prepare_history_envelope_output_recording(
                    phase,
                    &access,
                    reservation,
                    candidate,
                    self.receipt.receipt(),
                    forest.profile(),
                )
                .map_err(Denial::Adoption)?;
            forest.bind_origin(origin).map_err(Denial::Outputs)?;
        }
        let origin = Arc::clone(forest.origin().ok_or(Denial::Outputs(
            WorthQueryHistoryOutputDenial::NeedsDisposal,
        ))?);
        let installed = runtime
            .installed_schema()
            .installed_history::<H>()
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause))
            })?;
        let request = |branch| WorthQueryApplicationRequest {
            application: runtime,
            principal: self.principal,
            scope: self.scope,
            branch,
        };
        let on_candidate = request(candidate.branch());
        let complete = self
            .document
            .payload
            .with_read(
                request(owner.branch()).query(self.document.access.clone()),
                phase,
                None,
                |runtime, binding, external, plan| {
                    let read =
                        WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan);
                    Ok(runtime.with_document_history_envelope_recording_outputs(
                        phase,
                        &access,
                        owner,
                        read,
                        reservation,
                        candidate,
                        self.constraint,
                        &origin,
                        |authority| forest.advance(phase, &on_candidate, authority),
                    ))
                },
            )
            .map_err(|cause| Denial::History(WorthQueryApplicationHistoryDenial::Query(cause)))?
            .map_err(|cause| Denial::Outputs(WorthQueryHistoryOutputDenial::Navigation(cause)))?
            .map_err(Denial::Outputs)?;
        if complete {
            forest
                .drain_for_selection(phase, candidate.branch())
                .map_err(Denial::Outputs)?;
        }
        Ok(complete)
    }
}
