use super::{composition::UiAllocationStreamCommitDecision, UiAllocationFrameIngressRef};

mod accessors;
mod ordering;
mod posture;
mod source_order_transition;
use crate::evidence::{
    UiAllocationStreamPolicyDenialEvidenceReceipt, UiAllocationStreamPolicyEvidenceOutcome,
    UiAllocationStreamPolicyEvidenceReceipt, UiAllocationStreamPolicyPayloadCounters,
    UiMeasurementValue,
};
use crate::runtime::allocation_frame_dispatch::{
    UiAdmittedAllocationStreamFrame, UiAllocationFrameDuplicateWitness,
    UiAllocationFrameIngressKey, UiPendingAllocationFrameHandoff,
};
use crate::runtime::{
    UiAllocationFrameQueryWarningPosture, UiAllocationFrameSourceFact,
    UiAllocationInvalidationFamily, UiAllocationInvalidationIntent,
    UiAllocationStreamCompositionDenial, UiAllocationStreamFamily,
    UiResolvedAllocationStreamPolicy, WorthUiTransientInteractionState,
};
use posture::resolve_ingress_policy_verdict;
pub(crate) use source_order_transition::UiAllocationSourceOrderTransition;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationFrameResolutionDenial {
    SourceSequenceDuplicate { order: u64 },
    SourceSequenceRegression { previous: u64, observed: u64 },
    SourceSequenceGap { previous: u64, observed: u64 },
    UnsupportedSourcePosture,
    Policy(UiAllocationStreamCompositionDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationFrameCadenceVerdict {
    CommitEligible,
    PreviewOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationIngressPolicyVerdict {
    Current,
    PartialQueryStaleButBounded {
        warnings: UiAllocationFrameQueryWarningPosture,
        max_lag_frames: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationFrameResolutionCounters {
    entry_visits: u16,
    gap_checks: u16,
    policy_family_count: u8,
    invalidation_count: u16,
    order_ledger_scans: u16,
    order_ledger_writes: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationSourceOrderVerdict {
    FirstObserved,
    Contiguous,
    GapAccepted { missing: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationDuplicatePosture<'a> {
    witness: &'a UiAllocationFrameDuplicateWitness,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationFramePlanIdentity {
    evidence: UiAllocationStreamPolicyEvidenceOutcome,
}

#[derive(Debug, PartialEq)]
pub struct UiAllocationFrameRejection {
    evidence: UiAllocationStreamPolicyEvidenceOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationSourceOrderLedger {
    records: [Option<(u64, crate::runtime::UiAllocationFrameSourceGeneration, u64)>;
        crate::runtime::allocation_frame_dispatch::ALLOCATION_FRAME_SOURCE_CAPACITY],
}

#[derive(Debug, PartialEq)]
pub struct UiResolvedAllocationFramePlan {
    identity: UiAllocationFramePlanIdentity,
    counters: UiAllocationFrameResolutionCounters,
    sources: Box<[UiAllocationFrameSourceFact]>,
}

pub(crate) enum UiAllocationFrameConsumptionDisposition {
    Accepted {
        plan: UiResolvedAllocationFramePlan,
        source_order_transition: Box<UiAllocationSourceOrderTransition>,
    },
    Rejected(UiAllocationFrameRejection),
}

struct UiResolvedAllocationFrameCandidate {
    identity: UiAllocationFramePlanIdentity,
    counters: UiAllocationFrameResolutionCounters,
    accepted_order_ledger: UiAllocationSourceOrderLedger,
    sources: Box<[UiAllocationFrameSourceFact]>,
}

struct UiAllocationFrameResolutionFailure {
    epoch: crate::runtime::UiAllocationFrameEpoch,
    duplicate_witness: UiAllocationFrameDuplicateWitness,
    ingress: Box<[crate::runtime::UiAllocationFrameSourceFactPosture]>,
    denial: UiAllocationFrameResolutionDenial,
    order_verdicts: Box<[UiAllocationSourceOrderVerdict]>,
    ingress_policy_verdicts: Box<[UiAllocationIngressPolicyVerdict]>,
    payload_counters: UiAllocationStreamPolicyPayloadCounters,
}

pub(crate) fn consume_pending_frame(
    pending: UiPendingAllocationFrameHandoff,
    order_ledger: &UiAllocationSourceOrderLedger,
) -> UiAllocationFrameConsumptionDisposition {
    let frame = pending.into_sealed_frame();
    match resolve_frame(frame, order_ledger) {
        Ok(candidate) => {
            let UiResolvedAllocationFrameCandidate {
                identity,
                counters,
                accepted_order_ledger,
                sources,
            } = candidate;
            UiAllocationFrameConsumptionDisposition::Accepted {
                plan: UiResolvedAllocationFramePlan {
                    identity,
                    counters,
                    sources,
                },
                source_order_transition: Box::new(UiAllocationSourceOrderTransition::new(
                    order_ledger.clone(),
                    accepted_order_ledger,
                )),
            }
        }
        Err(UiAllocationFrameResolutionFailure {
            epoch,
            duplicate_witness,
            ingress,
            denial,
            order_verdicts,
            ingress_policy_verdicts,
            payload_counters,
        }) => {
            let evidence = UiAllocationStreamPolicyEvidenceOutcome::Denied(
                UiAllocationStreamPolicyDenialEvidenceReceipt::new(
                    epoch,
                    ingress,
                    duplicate_witness,
                    order_verdicts,
                    ingress_policy_verdicts,
                    denial,
                    payload_counters,
                ),
            );
            UiAllocationFrameConsumptionDisposition::Rejected(UiAllocationFrameRejection {
                evidence,
            })
        }
    }
}

fn resolve_frame(
    frame: UiAdmittedAllocationStreamFrame,
    order_ledger: &UiAllocationSourceOrderLedger,
) -> Result<UiResolvedAllocationFrameCandidate, UiAllocationFrameResolutionFailure> {
    let (epoch, entries, duplicate_witness) = frame.into_policy_input();
    let ingress = entries.view();
    let mut payload_counters = UiAllocationStreamPolicyPayloadCounters::default();
    payload_counters.reserve_vector_capacity(ingress.len());
    let mut families = Vec::with_capacity(ingress.len());
    payload_counters.reserve_vector_capacity(ingress.len());
    let mut invalidations = Vec::with_capacity(ingress.len());
    let mut gap_checks = 0_u16;
    let mut classification_denial = None;
    let mut order_denial = None;
    payload_counters.reserve_vector_capacity(ingress.len());
    let mut order_verdicts = Vec::with_capacity(ingress.len());
    payload_counters.reserve_vector_capacity(ingress.len());
    let mut ingress_policy_verdicts = Vec::with_capacity(ingress.len());
    let mut ledger_scans = 0_u16;
    let mut staged_order_ledger = order_ledger.clone();
    for (ingress_index, entry) in ingress.iter().enumerate() {
        let (family, invalidation) = match classify(entry.source_fact()) {
            Ok(classification) => classification,
            Err(denial) => {
                classification_denial = Some(denial);
                break;
            }
        };
        gap_checks = gap_checks.saturating_add(1);
        ledger_scans = ledger_scans.saturating_add(1);
        let verdict = match staged_order_ledger.evaluate_and_stage(entry, family) {
            Ok(verdict) => verdict,
            Err(denial) => {
                order_denial = Some(denial);
                break;
            }
        };
        order_verdicts.push(verdict);
        ingress_policy_verdicts.push(resolve_ingress_policy_verdict(entry.source_fact(), family));
        invalidations.push(UiAllocationInvalidationIntent::new(
            invalidation,
            UiAllocationFrameIngressRef::mint(epoch, ingress_index),
        ));
        families.push(family);
    }
    if let Some(denial) = classification_denial {
        return Err(UiAllocationFrameResolutionFailure {
            epoch,
            duplicate_witness,
            ingress: denial_postures(ingress, &mut payload_counters),
            denial,
            order_verdicts: into_counted_box(order_verdicts, &mut payload_counters),
            ingress_policy_verdicts: into_counted_box(
                ingress_policy_verdicts,
                &mut payload_counters,
            ),
            payload_counters,
        });
    }
    if let Some(denial) = order_denial {
        return Err(UiAllocationFrameResolutionFailure {
            epoch,
            duplicate_witness,
            ingress: denial_postures(ingress, &mut payload_counters),
            denial,
            order_verdicts: into_counted_box(order_verdicts, &mut payload_counters),
            ingress_policy_verdicts: into_counted_box(
                ingress_policy_verdicts,
                &mut payload_counters,
            ),
            payload_counters,
        });
    }
    let decision = super::composition::resolve_stream_families(&families, &mut payload_counters);
    let (receipt, cadence) = match decision {
        UiAllocationStreamCommitDecision::Commit(receipt) => {
            (receipt, UiAllocationFrameCadenceVerdict::CommitEligible)
        }
        UiAllocationStreamCommitDecision::Preview(receipt) => {
            (receipt, UiAllocationFrameCadenceVerdict::PreviewOnly)
        }
        UiAllocationStreamCommitDecision::Denied(denial) => {
            return Err(UiAllocationFrameResolutionFailure {
                epoch,
                duplicate_witness,
                ingress: denial_postures(ingress, &mut payload_counters),
                denial: UiAllocationFrameResolutionDenial::Policy(denial),
                order_verdicts: into_counted_box(order_verdicts, &mut payload_counters),
                ingress_policy_verdicts: into_counted_box(
                    ingress_policy_verdicts,
                    &mut payload_counters,
                ),
                payload_counters,
            })
        }
    };
    let counters = UiAllocationFrameResolutionCounters {
        entry_visits: u16::try_from(families.len())
            .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress"),
        gap_checks,
        policy_family_count: u8::try_from(receipt.families().len())
            .expect("a frame composes at most the eight stream families"),
        invalidation_count: u16::try_from(invalidations.len())
            .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress"),
        order_ledger_scans: ledger_scans,
        order_ledger_writes: u16::try_from(families.len())
            .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress"),
    };
    let invalidations = into_counted_box(invalidations, &mut payload_counters);
    let (policy, intermediate_policy_verdicts, policy_branches, composition_counters) =
        receipt.into_resolution_parts();
    let ingress_policy_verdicts = into_counted_box(ingress_policy_verdicts, &mut payload_counters);
    let evidence = UiAllocationStreamPolicyEvidenceOutcome::Resolved(
        UiAllocationStreamPolicyEvidenceReceipt::new(
            crate::evidence::UiAllocationStreamPolicyEvidenceInput {
                epoch,
                families: into_counted_box(families, &mut payload_counters),
                order_verdicts: into_counted_box(order_verdicts, &mut payload_counters),
                duplicate_witness,
                invalidations,
                policy,
                intermediate: intermediate_policy_verdicts,
                branches: policy_branches,
                ingress_policy_verdicts,
                cadence,
                composition_counters,
                payload_counters,
            },
        ),
    );
    let sources = entries
        .into_ingress()
        .into_vec()
        .into_iter()
        .map(|entry| entry.into_source_fact())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(UiResolvedAllocationFrameCandidate {
        identity: UiAllocationFramePlanIdentity { evidence },
        counters,
        accepted_order_ledger: staged_order_ledger,
        sources,
    })
}

fn into_counted_box<T>(
    values: Vec<T>,
    counters: &mut UiAllocationStreamPolicyPayloadCounters,
) -> Box<[T]> {
    counters.convert_boxed_slice();
    values.into_boxed_slice()
}

fn denial_postures(
    ingress: crate::runtime::allocation_frame_dispatch::UiAllocationFrameIngressView<'_>,
    counters: &mut UiAllocationStreamPolicyPayloadCounters,
) -> Box<[crate::runtime::UiAllocationFrameSourceFactPosture]> {
    counters.reserve_vector_capacity(ingress.len());
    let mut postures = Vec::with_capacity(ingress.len());
    for entry in ingress.iter() {
        counters.copy_denial_source_posture();
        postures.push(entry.descriptor().source_fact_posture());
    }
    into_counted_box(postures, counters)
}

#[rustfmt::skip]
fn classify(fact: &UiAllocationFrameSourceFact) -> Result<(UiAllocationStreamFamily, UiAllocationInvalidationFamily), UiAllocationFrameResolutionDenial> {
    Ok(match fact {
        UiAllocationFrameSourceFact::QuerySettledFact { fact, .. } => {
            let carries_content_extent = fact
                .measurement_facts()
                .observations()
                .iter()
                .any(|observation| {
                    observation.family()
                        == worth_ui_query_binding::WorthUiQueryMeasurementFactFamily::ScrollContentExtent
                });
            let invalidation = if carries_content_extent {
                UiAllocationInvalidationFamily::ContentExtentChange
            } else {
                UiAllocationInvalidationFamily::QueryMeasurementFactChange
            };
            (UiAllocationStreamFamily::QueryProjection, invalidation)
        },
        UiAllocationFrameSourceFact::DurableResize(_) =>
            (UiAllocationStreamFamily::DurableResize, UiAllocationInvalidationFamily::DurableLocalResizeChange),
        UiAllocationFrameSourceFact::Interaction(value) => match value.state() {
            WorthUiTransientInteractionState::TextInput =>
                (UiAllocationStreamFamily::TextInput, UiAllocationInvalidationFamily::TextContentChange),
            WorthUiTransientInteractionState::ResizePreview =>
                (UiAllocationStreamFamily::ResizePreview, UiAllocationInvalidationFamily::ResizePreviewDelta),
            _ => return Err(UiAllocationFrameResolutionDenial::UnsupportedSourcePosture),
        },
        UiAllocationFrameSourceFact::HostMeasurement(value) => match value.result().value() {
            UiMeasurementValue::ViewportExtent(_) =>
                (UiAllocationStreamFamily::ViewportObservation, UiAllocationInvalidationFamily::ViewportExtentChange),
            UiMeasurementValue::ScrollContainerViewport(_) =>
                (UiAllocationStreamFamily::ScrollExtentObservation, UiAllocationInvalidationFamily::ScrollExtentObservation),
            UiMeasurementValue::PortalAnchorRect(_) =>
                (UiAllocationStreamFamily::PortalAnchorObservation, UiAllocationInvalidationFamily::PortalAnchorMovement),
            _ => (UiAllocationStreamFamily::HostMeasurementReplacement, UiAllocationInvalidationFamily::HostMeasurementResultReplacement),
        },
    })
}
