use std::sync::Arc;

use crate::runtime::intent::{
    UiIntentAdmissionLease, UiIntentAdmissionSlotIdentity, UiIntentAttemptLineage,
    UiIntentOccupancyPlacement,
};

use super::{
    UiIntentExecutionAdmissionCommit, UiIntentExecutionAdmissionReservationFailure,
    UiIntentExecutionAdmissionReservationFailureReason, UiIntentExecutionSlotPhase,
    UiIntentExecutionState, UiReservedIntentAdmission,
};
use crate::runtime::intent_execution::{
    UiIntentExecutionReservationBasis, UiIntentExecutionReservationCounts,
};

struct UiIntentExecutionReservationPlan {
    basis: UiIntentExecutionReservationBasis,
    execution_slot: UiIntentExecutionSlotPlacement,
    occupancy: UiIntentOccupancyPlacement,
    occupancy_slots_inspected: usize,
    definition: crate::capability::UiIntentId,
    declaration: crate::declaration::UiIntentDeclarationIdentity,
    lease: Arc<UiIntentAdmissionLease>,
    consequence_basis: super::reservation_authority::UiReservedIntentConsequenceBasis,
}

struct UiIntentExecutionSlotPlacement {
    index: usize,
    generation: u64,
    slots_inspected: usize,
}

impl UiIntentExecutionState {
    pub(crate) fn reserve_admission(
        &mut self,
        candidate: crate::runtime::intent::UiCurrentIntentAdmissionCandidate,
        lineage: UiIntentAttemptLineage,
    ) -> Result<UiIntentExecutionAdmissionCommit, UiIntentExecutionAdmissionReservationFailure>
    {
        let plan = self.prepare_reservation(&candidate)?;
        Ok(self.commit_reservation(candidate, lineage, plan))
    }

    fn prepare_reservation(
        &self,
        candidate: &crate::runtime::intent::UiCurrentIntentAdmissionCandidate,
    ) -> Result<UiIntentExecutionReservationPlan, UiIntentExecutionAdmissionReservationFailure>
    {
        let basis = candidate.execution_reservation_basis();
        self.capacity
            .admit(basis, self.reservation_counts(basis))
            .map_err(|denial| UiIntentExecutionAdmissionReservationFailure {
                reason: UiIntentExecutionAdmissionReservationFailureReason::Capacity(denial),
                slots_inspected: self.slots.len(),
                occupancy_slots_inspected: 0,
            })?;
        let execution_slot = self.prepare_execution_slot().ok_or(
            UiIntentExecutionAdmissionReservationFailure {
                reason:
                    UiIntentExecutionAdmissionReservationFailureReason::ReservationIdentityExhausted,
                slots_inspected: self.slots.len(),
                occupancy_slots_inspected: 0,
            },
        )?;
        let occupancy = self
            .occupancy
            .prepare_candidate_reservation(candidate)
            .map_err(|failure| UiIntentExecutionAdmissionReservationFailure {
                reason: UiIntentExecutionAdmissionReservationFailureReason::Occupancy(
                    failure.denial(),
                ),
                slots_inspected: execution_slot.slots_inspected,
                occupancy_slots_inspected: failure.slots_inspected(),
            })?;
        Ok(UiIntentExecutionReservationPlan {
            basis,
            occupancy_slots_inspected: occupancy.slots_inspected(),
            execution_slot,
            occupancy,
            definition: candidate.definition_id(),
            declaration: candidate.declaration_identity_value().clone(),
            lease: Arc::new(UiIntentAdmissionLease::new()),
            consequence_basis: super::reservation_authority::UiReservedIntentConsequenceBasis {
                graph_node: candidate.graph_node(),
                target: candidate.target(),
                generation: candidate.generation().clone(),
                declaration: Arc::clone(candidate.declaration_reference()),
                portal_declaration: candidate.portal_declaration(),
                selection_option: candidate.selection_option().cloned(),
                command_route: candidate
                    .command_route_receipt()
                    .map(crate::runtime::UiCommandRouteReceipt::evidence),
            },
        })
    }

    fn commit_reservation(
        &mut self,
        candidate: crate::runtime::intent::UiCurrentIntentAdmissionCandidate,
        lineage: UiIntentAttemptLineage,
        plan: UiIntentExecutionReservationPlan,
    ) -> UiIntentExecutionAdmissionCommit {
        let occupancy = self
            .occupancy
            .commit_candidate_reservation(&candidate, plan.occupancy);
        let slot_identity = UiIntentAdmissionSlotIdentity::new(
            plan.execution_slot.index as u8,
            plan.execution_slot.generation,
        );
        self.slots[plan.execution_slot.index].generation = plan.execution_slot.generation;
        self.slots[plan.execution_slot.index].phase = Some(UiIntentExecutionSlotPhase::Admitted(
            UiReservedIntentAdmission {
                reservation: super::UiReservedIntentExecutionReservation {
                    core: super::UiIntentExecutionReservationCore {
                        target: candidate.target(),
                        occupancy,
                        basis: plan.basis,
                        lineage,
                        lease: Arc::clone(&plan.lease),
                        retained_payloads: candidate.retained_payload_count(),
                        retained_owner_references: candidate.retained_owner_reference_count(),
                    },
                    consequence_basis: plan.consequence_basis,
                },
                candidate,
            },
        ));
        UiIntentExecutionAdmissionCommit {
            identity: crate::runtime::intent::UiAdmittedIntentIdentity::new(
                slot_identity,
                lineage,
                plan.definition,
                plan.declaration,
            ),
            lease: plan.lease,
            slots_inspected: plan.execution_slot.slots_inspected,
            occupancy_slots_inspected: plan.occupancy_slots_inspected,
        }
    }

    fn prepare_execution_slot(&self) -> Option<UiIntentExecutionSlotPlacement> {
        for (index, slot) in self.slots.iter().enumerate() {
            if slot.phase.is_some() {
                continue;
            }
            if let Some(generation) = slot.generation.checked_add(1) {
                return Some(UiIntentExecutionSlotPlacement {
                    index,
                    generation,
                    slots_inspected: index + 1,
                });
            }
        }
        None
    }

    fn reservation_counts(
        &self,
        requested: UiIntentExecutionReservationBasis,
    ) -> UiIntentExecutionReservationCounts {
        let mut counts = UiIntentExecutionReservationCounts::default();
        for active in self
            .slots
            .iter()
            .filter_map(|slot| slot.phase.as_ref()?.reservation())
            .map(|reservation| reservation.basis)
        {
            counts.application_attempts += 1;
            counts.retained_payload_bytes += active.retained_payload_bytes();
            counts.destination_attempts +=
                usize::from(active.destination() == requested.destination());
            counts.provider_attempts += usize::from(active.same_provider(requested));
            counts.intent_attempts += usize::from(active.intent() == requested.intent());
        }
        counts
    }
}
