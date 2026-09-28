use crate::admission::{UiMeasurementAdmission, UiSupportSnapshot};
use crate::declaration::stable_text_digest;
use crate::obligations::selection::UiSelectedObligationSet;
use crate::obligations::verdict::{
    UiObligationDispatchStopPosture, UiObligationVerdict, UiObligationVerdictClass,
};

use super::{dispatch_execution::UiObligationDispatchExecution, UiObligationDispatchEntry};
use crate::obligations::catalog::UiObligationFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiObligationDispatchPlan {
    selected: UiSelectedObligationSet,
    support_snapshot: UiSupportSnapshot,
    measurement_admission: Option<UiMeasurementAdmission>,
    entries: Box<[UiObligationDispatchEntry]>,
    plan_stop_posture: UiObligationDispatchStopPosture,
    shape_digest: u64,
}

impl UiObligationDispatchPlan {
    pub(crate) fn new(
        selected: UiSelectedObligationSet,
        support_snapshot: UiSupportSnapshot,
        measurement_admission: Option<UiMeasurementAdmission>,
        entries: Box<[UiObligationDispatchEntry]>,
        plan_stop_posture: UiObligationDispatchStopPosture,
    ) -> Self {
        let shape_digest = entries.iter().fold(
            stable_text_digest("obligation-dispatch-plan")
                ^ selected.touch().identity_digest().rotate_left(7)
                ^ stable_text_digest(&format!("{:?}", support_snapshot.posture())).rotate_left(11)
                ^ stable_text_digest(&format!("{measurement_admission:?}")).rotate_left(19)
                ^ digest_stop_posture(plan_stop_posture).rotate_left(13),
            |digest, entry| {
                digest
                    ^ entry
                        .selected()
                        .identity()
                        .identity_digest()
                        .rotate_left(17)
                    ^ (entry.selected().check_kind() as u64).rotate_left(23)
            },
        );

        Self {
            selected,
            support_snapshot,
            measurement_admission,
            entries,
            plan_stop_posture,
            shape_digest,
        }
    }

    pub fn selected(&self) -> &UiSelectedObligationSet {
        &self.selected
    }

    pub fn support_snapshot(&self) -> &UiSupportSnapshot {
        &self.support_snapshot
    }

    pub fn measurement_admission(&self) -> Option<&UiMeasurementAdmission> {
        self.measurement_admission.as_ref()
    }

    pub fn entries(&self) -> &[UiObligationDispatchEntry] {
        &self.entries
    }

    pub fn plan_stop_posture(&self) -> UiObligationDispatchStopPosture {
        self.plan_stop_posture
    }

    pub fn shape_digest(&self) -> u64 {
        self.shape_digest
    }

    pub fn execute(&self) -> Box<[UiObligationVerdict]> {
        if self.entries.is_empty()
            && self.plan_stop_posture != UiObligationDispatchStopPosture::None
        {
            return vec![UiObligationVerdict::global_stop(
                self.shape_digest,
                verdict_class_for_stop(self.plan_stop_posture),
                self.plan_stop_posture,
            )]
            .into_boxed_slice();
        }

        self.entries
            .iter()
            .map(|entry| {
                let selected = entry.selected();
                if self.plan_stop_posture != UiObligationDispatchStopPosture::None {
                    UiObligationVerdict::from_selected(
                        selected,
                        verdict_class_for_stop(self.plan_stop_posture),
                        self.plan_stop_posture,
                    )
                } else {
                    match entry.execution() {
                        UiObligationDispatchExecution::ImmediateCheck => match selected.family() {
                            UiObligationFamily::DiagnosticSurfaceRequirement => {
                                UiObligationVerdict::from_selected(
                                    selected,
                                    UiObligationVerdictClass::Advisory,
                                    UiObligationDispatchStopPosture::DiagnosticOnly,
                                )
                            }
                            UiObligationFamily::StructuralLegality
                            | UiObligationFamily::ParticipationLegality
                            | UiObligationFamily::SlotContract
                            | UiObligationFamily::MeasurementRequirement
                            | UiObligationFamily::QueryBindingRequirement
                            | UiObligationFamily::IntentOperabilityRequirement
                            | UiObligationFamily::PortalHostRequirement
                            | UiObligationFamily::FocusRouteRequirement
                            | UiObligationFamily::MotionSupportRequirement
                            | UiObligationFamily::ScrollRoutingRequirement
                            | UiObligationFamily::SelectionStateRequirement
                            | UiObligationFamily::CommandRouteRequirement
                            | UiObligationFamily::AccessibilityRequirement
                            | UiObligationFamily::HostCapabilityRequirement => {
                                UiObligationVerdict::from_selected(
                                    selected,
                                    UiObligationVerdictClass::Success,
                                    UiObligationDispatchStopPosture::None,
                                )
                            }
                        },
                        UiObligationDispatchExecution::TypedStop(stop_posture) => {
                            UiObligationVerdict::from_selected(
                                selected,
                                UiObligationVerdictClass::Advisory,
                                stop_posture,
                            )
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

const fn verdict_class_for_stop(
    stop_posture: UiObligationDispatchStopPosture,
) -> UiObligationVerdictClass {
    match stop_posture {
        UiObligationDispatchStopPosture::None => UiObligationVerdictClass::Success,
        UiObligationDispatchStopPosture::DiagnosticOnly => UiObligationVerdictClass::Advisory,
        UiObligationDispatchStopPosture::Unsupported
        | UiObligationDispatchStopPosture::Deferred
        | UiObligationDispatchStopPosture::WrongWorld
        | UiObligationDispatchStopPosture::WrongQueryBasis { .. }
        | UiObligationDispatchStopPosture::WrongHostCapability { .. }
        | UiObligationDispatchStopPosture::Stale { .. }
        | UiObligationDispatchStopPosture::Ambiguous { .. }
        | UiObligationDispatchStopPosture::BudgetExceeded { .. } => {
            UiObligationVerdictClass::Violation
        }
    }
}

fn digest_stop_posture(stop_posture: UiObligationDispatchStopPosture) -> u64 {
    match stop_posture {
        UiObligationDispatchStopPosture::None => 0,
        UiObligationDispatchStopPosture::Unsupported => 1,
        UiObligationDispatchStopPosture::Deferred => 2,
        UiObligationDispatchStopPosture::DiagnosticOnly => 3,
        UiObligationDispatchStopPosture::WrongWorld => 4,
        UiObligationDispatchStopPosture::WrongQueryBasis { required, observed } => {
            5 ^ (required as u64).rotate_left(7) ^ (observed as u64).rotate_left(13)
        }
        UiObligationDispatchStopPosture::WrongHostCapability { required, observed } => {
            6 ^ (required as u64).rotate_left(7) ^ (observed as u64).rotate_left(13)
        }
        UiObligationDispatchStopPosture::Stale {
            required,
            observed,
            evidence,
        } => {
            7 ^ (required as u64).rotate_left(7)
                ^ (observed as u64).rotate_left(13)
                ^ (evidence as u64).rotate_left(19)
        }
        UiObligationDispatchStopPosture::Ambiguous {
            required_query_basis,
            observed_query_basis,
            required_host_capability,
            observed_host_capability,
        } => {
            8 ^ required_query_basis
                .map(|value| value as u64)
                .unwrap_or(0)
                .rotate_left(7)
                ^ observed_query_basis
                    .map(|value| value as u64)
                    .unwrap_or(0)
                    .rotate_left(13)
                ^ required_host_capability
                    .map(|value| value as u64)
                    .unwrap_or(0)
                    .rotate_left(19)
                ^ observed_host_capability
                    .map(|value| value as u64)
                    .unwrap_or(0)
                    .rotate_left(23)
        }
        UiObligationDispatchStopPosture::BudgetExceeded {
            budget,
            attempted_lane_cost,
        } => {
            let budget_digest = match budget {
                crate::admission::UiAdmissionSelectionBudget::Unbounded => 0,
                crate::admission::UiAdmissionSelectionBudget::OrdinaryLaneBudget { lane_limit } => {
                    lane_limit as u64
                }
            };
            9 ^ budget_digest ^ (attempted_lane_cost as u64).rotate_left(7)
        }
    }
}
