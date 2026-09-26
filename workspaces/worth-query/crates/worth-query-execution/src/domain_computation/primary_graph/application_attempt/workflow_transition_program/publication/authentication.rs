use std::sync::Arc;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
use worth_query_admission::facade::{
    authenticated_principal::{WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope},
    authentication_event::{
        WorthQueryAuthenticationEvent, WorthQueryAuthenticationEventDenial,
        WorthQueryAuthenticationEventIntent, WorthQueryAuthenticationEventSigningOwner,
        WorthQueryConsumedAuthenticationEvent,
    },
};

use super::{PreparedWorkflowAdvance, PreparedWorkflowTransitionReplays};

struct ApprovalDescriptor {
    program_revision: ApplicationProgramRevision,
    transition_identity: String,
    transition_identity_bytes: [u8; 32],
    transition_identity_locator: AspectFieldLocator,
    assessment_identity_locator: AspectFieldLocator,
    instance: EntityId,
    node_path: String,
    approval_identity: [u8; 32],
    projection_identity: String,
    progress_update: PreparedWorkflowProgressUpdate,
    replays: Option<PreparedWorkflowTransitionReplays>,
}

fn matches_outer_approval_identity(
    expected: (&str, EntityId, &str, &[u8; 32], &str),
    presented: (&str, EntityId, &str, Option<&[u8; 32]>, Option<&str>),
) -> bool {
    expected.0 == presented.0
        && expected.1 == presented.1
        && expected.2 == presented.2
        && presented.3 == Some(expected.3)
        && presented.4 == Some(expected.4)
}

/// Only Query's approval preparation can attach this move-only authentication
/// proof; the commit owner rechecks it after replay resolution.
#[doc(hidden)]
pub struct PreparedWorkflowApprovalAuthentication<Schema> {
    owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
    intent: WorthQueryAuthenticationEventIntent,
    basis: Arc<[WorthQueryApplicationObservedFact]>,
    descriptor: ApprovalDescriptor,
    consumed: Option<WorthQueryConsumedAuthenticationEvent<Schema>>,
}

impl<Schema> PreparedWorkflowApprovalAuthentication<Schema> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn pending<
        Operation,
        Input,
        Scope,
    >(
        owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
        intent: WorthQueryAuthenticationEventIntent,
        basis: Arc<[WorthQueryApplicationObservedFact]>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    ) -> Self {
        let PreparedWorkflowAdvance::Transition {
            program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator,
            assessment_identity_locator,
            instance,
            node_path,
            terminal: false,
            assessment: None,
            supporting_identity: None,
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            approval: Some(approval),
            approval_identity: Some(approval_identity),
            ..
        } = prepared
        else {
            unreachable!("Query only seals its materialized approval")
        };
        Self {
            owner,
            intent,
            basis,
            descriptor: ApprovalDescriptor {
                program_revision: program_revision.clone(),
                transition_identity: transition_identity.clone(),
                transition_identity_bytes: *transition_identity_bytes,
                transition_identity_locator: transition_identity_locator.clone(),
                assessment_identity_locator: assessment_identity_locator.clone(),
                instance: *instance,
                node_path: node_path.clone(),
                approval_identity: *approval_identity,
                projection_identity: approval.identity.clone(),
                progress_update: progress_update.clone(),
                replays: None,
            },
            consumed: None,
        }
    }

    pub(super) fn intent(&self) -> &WorthQueryAuthenticationEventIntent {
        &self.intent
    }

    pub(super) fn matches_basis(
        &self,
        candidate: Option<&Arc<[WorthQueryApplicationObservedFact]>>,
    ) -> bool {
        candidate.is_some_and(|candidate| Arc::ptr_eq(&self.basis, candidate))
    }

    pub(super) fn bind_replays(&mut self, replays: &PreparedWorkflowTransitionReplays) {
        self.descriptor.replays = Some(replays.clone());
    }

    pub(super) fn trusted_replays(&self) -> Option<&PreparedWorkflowTransitionReplays> {
        self.descriptor.replays.as_ref()
    }

    pub(super) fn trusted_progress_update(&self) -> PreparedWorkflowProgressUpdate {
        self.descriptor.progress_update.clone()
    }

    pub(super) fn matches_descriptor<Operation, Input, Scope>(
        &self,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    ) -> bool {
        let PreparedWorkflowAdvance::Transition {
            program,
            program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator,
            assessment_identity_locator,
            instance,
            node_path,
            terminal,
            assessment,
            supporting_identity,
            operation_receipt_identity,
            approval,
            approval_identity,
            ..
        } = prepared
        else {
            return false;
        };
        let expected = &self.descriptor;
        self.matches_basis(program.output_currentness_facts.as_ref())
            && program_revision == &expected.program_revision
            && transition_identity_bytes == &expected.transition_identity_bytes
            && transition_identity_locator == &expected.transition_identity_locator
            && assessment_identity_locator == &expected.assessment_identity_locator
            && matches_outer_approval_identity(
                (
                    &expected.transition_identity,
                    expected.instance,
                    &expected.node_path,
                    &expected.approval_identity,
                    &expected.projection_identity,
                ),
                (
                    transition_identity,
                    *instance,
                    node_path,
                    approval_identity.as_ref(),
                    approval.as_ref().map(|approval| approval.identity.as_str()),
                ),
            )
            && !terminal
            && assessment.is_none()
            && supporting_identity.is_none()
            && operation_receipt_identity.is_none()
            && expected.replays.is_some()
    }

    pub(super) fn sign(
        mut self,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, WorthQueryAuthenticationEventDenial> {
        self.consumed =
            Some(
                self.owner
                    .consume_for_signing(event, principal, &self.intent, scope)?,
            );
        Ok(self)
    }

    pub(super) fn readmit(self) -> Result<(), WorthQueryAuthenticationEventDenial> {
        let consumed = self
            .consumed
            .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
        self.owner.readmit_for_publication(consumed).map(|_| ())
    }
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    pub(super) fn validate_approval_descriptor(
        &self,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        match self {
            Self::Transition {
                approval: Some(_),
                approval_authentication: Some(authentication),
                ..
            } if authentication.matches_descriptor(self) => Ok(()),
            Self::Transition {
                approval: Some(_), ..
            }
            | Self::Transition {
                approval_authentication: Some(_),
                ..
            } => Err(WorthQueryAuthenticationEventDenial::MissingSigningProof),
            _ => Ok(()),
        }
    }

    pub fn approval_authentication_intent(&self) -> Option<&WorthQueryAuthenticationEventIntent> {
        match self {
            Self::Transition {
                approval: Some(_),
                approval_authentication: Some(authentication),
                ..
            } => Some(authentication.intent()),
            _ => None,
        }
    }

    pub fn sign_approval(
        mut self,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, WorthQueryAuthenticationEventDenial> {
        self.validate_approval_descriptor()?;
        match &mut self {
            Self::Transition {
                approval: Some(_),
                approval_authentication,
                ..
            } => {
                let pending = approval_authentication
                    .take()
                    .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
                *approval_authentication = Some(pending.sign(event, principal, scope)?);
            }
            Self::ReplayOnly { .. } => {}
            _ => return Err(WorthQueryAuthenticationEventDenial::MissingSigningProof),
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::identity::PartitionId;

    use super::*;

    #[test]
    fn combined_program_and_proof_move_cannot_relabel_an_approval() {
        // A caller can move a genuine program together with its sealed proof.
        // The descriptor must still reject a different transition, instance,
        // node, approval, or projection outside that pair.
        let instance = EntityId::new(PartitionId::main(), 1, 0);
        let other_instance = EntityId::new(PartitionId::main(), 2, 0);
        let approval = [7; 32];
        let other_approval = [8; 32];
        let expected = (
            "transition-a",
            instance,
            "approve",
            &approval,
            "projection-a",
        );
        assert!(matches_outer_approval_identity(
            expected,
            (
                "transition-a",
                instance,
                "approve",
                Some(&approval),
                Some("projection-a")
            ),
        ));
        for swapped in [
            (
                "transition-b",
                instance,
                "approve",
                Some(&approval),
                Some("projection-a"),
            ),
            (
                "transition-a",
                other_instance,
                "approve",
                Some(&approval),
                Some("projection-a"),
            ),
            (
                "transition-a",
                instance,
                "other-node",
                Some(&approval),
                Some("projection-a"),
            ),
            (
                "transition-a",
                instance,
                "approve",
                Some(&other_approval),
                Some("projection-a"),
            ),
            (
                "transition-a",
                instance,
                "approve",
                Some(&approval),
                Some("projection-b"),
            ),
        ] {
            assert!(!matches_outer_approval_identity(expected, swapped));
        }
    }
}
