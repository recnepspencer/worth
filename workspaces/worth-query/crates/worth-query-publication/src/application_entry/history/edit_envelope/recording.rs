//! One reservation, one private candidate, one commit and one adoption.
use super::*;
use crate::application_entry::correction::WorthQueryApplicationCorrectionDenial;
use crate::application_entry::mutation::envelope::{
    runtime::{CapturedEnvelopeIntent, CorrectionSource},
    EnvelopeContext, WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeOutcome,
};
use std::{ops::ControlFlow, sync::Arc};
use worth_query_execution::{
    facade::{product::WorthQueryExpectedProductBranch, runtime::ExecutionAllocationPolicy},
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryCorrectionMeaning, WorthQueryDocumentHistoryEnvelopeConstraint,
        WorthQueryDocumentHistoryEnvelopeIntent,
        WorthQueryDocumentHistoryRecordPreparation, WorthQueryDocumentHistoryReservation,
        WorthQueryHistoryStepMetadata,
    },
};
use WorthQueryHistoryEditEnvelopeDenial as Denial;

/// What the history owner decided before any stage was admitted.
enum Admission<'runtime, S> {
    /// The envelope refused before it consulted the history owner.
    Unreached,
    Refused(Denial),
    AlreadyRecorded(WorthQueryHistoryStep),
    /// A record is reserved, but the stages cannot run.
    Stopped {
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: Option<WorthQueryExpectedProductBranch<'runtime, S>>,
        denial: Denial,
    },
    Running {
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        constraint: Arc<WorthQueryDocumentHistoryEnvelopeConstraint>,
    },
}

impl<'runtime, S, H, P, K, Payload, Outputs>
    WorthQueryHistoryEditEnvelope<'runtime, '_, '_, S, H, P, K, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    K: serde::Serialize + Send + Sync + 'static,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Execute every stage as one commit and record it as one step.
    ///
    /// This call is one-shot. It either records the step, names the step an
    /// earlier call already recorded, or returns the document at its prior
    /// head with nothing pending. `metadata` is stored with the step.
    ///
    /// A document with bound output trees records every tree at the commit
    /// before the head moves. The call drives their producers until all of
    /// them are recorded or the request can no longer advance.
    pub fn execute_recorded(
        self,
        metadata: WorthQueryHistoryStepMetadata,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryHistoryEditEnvelopeOutcome<'runtime, S, H, Payload, Outputs>,
        WorthQueryHistoryEditEnvelopeFailure<'runtime, S, H, Payload, Outputs>,
    > {
        let Self {
            document,
            envelope,
            principal,
            scope,
        } = self;
        let refuse = |document, denial| Err(WorthQueryHistoryEditEnvelopeFailure { document, denial });
        if !envelope.has_stages() {
            return refuse(document, Denial::Empty);
        }
        let mut admission = Admission::Unreached;
        let navigation = document.owner.correction_navigation_lineage();
        let executed = envelope.execute_with_history_front(allocation, Some(&*navigation), |context, phase, captured, correction| {
            admission = admit(&document, context, phase, captured, correction, &metadata);
            match &admission {
                Admission::Running {
                    reservation,
                    candidate,
                    constraint,
                } => {
                    context.branch = candidate.branch();
                    context.history = Some(constraint.clone());
                    context.correction_attempt = Some(reservation.attempt_identity());
                    ControlFlow::Continue(())
                }
                _ => ControlFlow::Break(()),
            }
        });
        let (reservation, candidate, constraint) = match admission {
            Admission::Unreached => {
                let denial = match executed {
                    Ok(outcome) => Denial::Envelope(outcome),
                    Err(cause) => execution(cause),
                };
                return refuse(document, denial);
            }
            Admission::Refused(denial) => return refuse(document, denial),
            Admission::AlreadyRecorded(step) => {
                return Ok(WorthQueryHistoryEditEnvelopeOutcome::AlreadyRecorded { document, step })
            }
            Admission::Stopped {
                reservation,
                candidate,
                denial,
            } => {
                abandon(&document, scope, reservation, candidate);
                return refuse(document, denial);
            }
            Admission::Running {
                reservation,
                candidate,
                constraint,
            } => (reservation, candidate, constraint),
        };
        let receipt = match executed {
            Ok(WorthQueryEditEnvelopeOutcome::Committed(receipt)) => receipt,
            uncommitted => {
                abandon(&document, scope, reservation, Some(candidate));
                let denial = match (constraint.take_refusal(), uncommitted) {
                    (Some((ordinal, denial)), _) => Denial::Stage { ordinal, denial },
                    (None, Ok(outcome)) => Denial::Envelope(outcome),
                    (None, Err(cause)) => execution(cause),
                };
                return refuse(document, denial);
            }
        };
        let sealed = sealing::Committed {
            document: &document,
            principal,
            scope,
            constraint: &constraint,
            receipt: &receipt,
        }
        .seal((reservation, candidate));
        match sealed {
            Ok(step) => Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded {
                document,
                step,
                receipt,
            }),
            Err(denial) => refuse(document, denial),
        }
    }
}

/// A correction's refusal stays typed; any other execution denial is the
/// envelope's.
fn execution(cause: WorthQueryEditEnvelopeExecutionDenial) -> Denial {
    match cause.into_correction() {
        Ok(denial) => Denial::Correction(denial),
        Err(cause) => Denial::Execution(cause),
    }
}

/// Read the document, reserve its one record and fork its private candidate.
/// A correction is refused only once the head is confirmed, so a diverged
/// document says `HeadDiverged` whatever its correction would have said;
/// its source must be a step on the document's selected line.
fn admit<'runtime, S, H, Payload, Outputs>(
    document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    context: &EnvelopeContext<'_, S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    captured: CapturedEnvelopeIntent,
    correction: Option<Result<CorrectionSource, WorthQueryApplicationCorrectionDenial>>,
    metadata: &WorthQueryHistoryStepMetadata,
) -> Admission<'runtime, S>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    let access = program_publication_access();
    let current = WorthQueryApplicationRequest {
        application: document.owner.application(),
        principal: context.principal,
        scope: context.scope,
        branch: document.owner.branch(),
    };
    if let Err(cause) = document.read_in_advancement(phase, &current) {
        return Admission::Refused(Denial::History(cause));
    }
    let correction = match correction {
        None => None,
        Some(Err(denial)) => return Admission::Refused(Denial::Correction(denial)),
        Some(Ok(CorrectionSource { binding, commit })) => {
            match document.owner.selected_line_correction_source(&commit) {
                Ok(source) => Some(WorthQueryDocumentHistoryCorrectionMeaning { binding, source }),
                Err(denial) => {
                    return Admission::Refused(Denial::History(
                        WorthQueryApplicationHistoryDenial::Document(denial),
                    ))
                }
            }
        }
    };
    let prepared = super::super::metadata::with_metadata_read::<S, H, _>(
        phase,
        &current,
        document.access.clone(),
        &document.payload,
        |_, installed, read| {
            let envelope = WorthQueryDocumentHistoryEnvelopeIntent::new(
                &captured.key,
                captured.intent,
                &captured.first_operation,
            );
            document.owner.reserve_history_envelope(
                phase,
                &access,
                installed,
                match correction {
                    Some(correction) => envelope.with_correction(correction),
                    None => envelope,
                },
                metadata,
                read,
            )
        },
    );
    let reservation = match prepared {
        Ok(WorthQueryDocumentHistoryRecordPreparation::Reserved(reservation)) => reservation,
        Ok(WorthQueryDocumentHistoryRecordPreparation::AlreadyRecorded(step)) => {
            return Admission::AlreadyRecorded(step)
        }
        // The step is named by its first stage, so that stage must be enrolled.
        Err(WorthQueryApplicationHistoryDenial::Document(
            denial @ WorthQueryDocumentHistoryDenial::OperationNotDeclared,
        )) => return Admission::Refused(Denial::Stage { ordinal: 0, denial }),
        Err(cause) => return Admission::Refused(Denial::History(cause)),
    };
    if let Err(cause) = document.owner.validate_record_delivery(phase, &access) {
        return Admission::Stopped {
            reservation,
            candidate: None,
            denial: Denial::History(WorthQueryApplicationHistoryDenial::Document(cause)),
        };
    }
    let candidate = match document
        .owner
        .create_history_candidate(phase, &access, &reservation)
    {
        Ok(candidate) => candidate,
        Err(cause) => {
            return Admission::Stopped {
                reservation,
                candidate: None,
                denial: Denial::Candidate(cause),
            }
        }
    };
    match document
        .owner
        .history_envelope_constraint(&access, &reservation, &candidate)
    {
        Ok(constraint) => Admission::Running {
            reservation,
            candidate,
            constraint,
        },
        Err(cause) => Admission::Stopped {
            reservation,
            candidate: Some(candidate),
            denial: Denial::History(WorthQueryApplicationHistoryDenial::Document(cause)),
        },
    }
}

/// Release the reservation and close the candidate in a phase of its own. A
/// request that can no longer advance leaves the close to retention.
fn abandon<'runtime, S, H, Payload, Outputs>(
    document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    scope: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    reservation: WorthQueryDocumentHistoryReservation,
    candidate: Option<WorthQueryExpectedProductBranch<'runtime, S>>,
) where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    let access = program_publication_access();
    let mut owners = Some((reservation, candidate));
    // A refused phase is not an error here: the rollback below runs without it.
    let _ = document
        .owner
        .application()
        .with_application_advancement(scope, |phase| {
            let (reservation, candidate) = owners.take().expect("the phase runs once");
            document
                .owner
                .abandon_history_envelope(Some(&phase), &access, reservation, candidate);
        });
    if let Some((reservation, candidate)) = owners {
        document
            .owner
            .abandon_history_envelope(None, &access, reservation, candidate);
    }
}
