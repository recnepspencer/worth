use std::sync::Arc;

use crate::runtime::intent::{
    UiIntentAdmissionLease, UiIntentAttemptLineage, UiIntentOccupancyReservation,
};
use crate::runtime::intent_execution::UiIntentExecutionReservationBasis;

pub(super) struct UiIntentExecutionReservationCore {
    pub(super) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(super) occupancy: UiIntentOccupancyReservation,
    pub(super) basis: UiIntentExecutionReservationBasis,
    pub(super) lineage: UiIntentAttemptLineage,
    pub(super) lease: Arc<UiIntentAdmissionLease>,
    pub(super) retained_payloads: usize,
    pub(super) retained_owner_references: usize,
}

pub(super) struct UiReservedIntentExecutionReservation {
    pub(super) core: UiIntentExecutionReservationCore,
    pub(super) consequence_basis: UiReservedIntentConsequenceBasis,
}

pub(super) struct UiActiveIntentExecutionReservation {
    pub(super) core: UiIntentExecutionReservationCore,
    consequence_basis: UiIntentConsequenceBasis,
}

pub(super) struct UiReservedIntentConsequenceBasis {
    pub(super) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(super) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(super) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) declaration: Arc<crate::declaration::UiCanonicalIntentDeclaration>,
    pub(super) portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    pub(super) selection_option: Option<worth_ui_query_binding::UiProjectionOptionReference>,
    pub(super) command_route: Option<crate::runtime::command_routing::UiCommandRouteEvidence>,
}

#[derive(Clone)]
pub(super) struct UiIntentConsequenceBasis {
    pub(super) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(super) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(super) target_affinity:
        crate::runtime::interaction::targeting::UiIntentExecutionTargetAffinity,
    pub(super) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) declaration: Arc<crate::declaration::UiCanonicalIntentDeclaration>,
    pub(super) portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    pub(super) selection_option: Option<worth_ui_query_binding::UiProjectionOptionReference>,
    pub(super) command_route: Option<crate::runtime::command_routing::UiCommandRouteEvidence>,
}

impl UiReservedIntentExecutionReservation {
    pub(super) fn activate(
        self,
        target_affinity: crate::runtime::interaction::targeting::UiIntentExecutionTargetAffinity,
    ) -> UiActiveIntentExecutionReservation {
        let UiReservedIntentConsequenceBasis {
            graph_node,
            target,
            generation,
            declaration,
            portal_declaration,
            selection_option,
            command_route,
        } = self.consequence_basis;
        UiActiveIntentExecutionReservation {
            core: self.core,
            consequence_basis: UiIntentConsequenceBasis {
                graph_node,
                target,
                target_affinity,
                generation,
                declaration,
                portal_declaration,
                selection_option,
                command_route,
            },
        }
    }
}

impl UiActiveIntentExecutionReservation {
    pub(super) fn consequence_basis(&self) -> UiIntentConsequenceBasis {
        self.consequence_basis.clone()
    }

    pub(super) fn posture_basis(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionPostureBasis {
        crate::runtime::intent_execution::UiIntentExecutionPostureBasis {
            graph_node: self.consequence_basis.graph_node,
            target: self.consequence_basis.target,
        }
    }
}
