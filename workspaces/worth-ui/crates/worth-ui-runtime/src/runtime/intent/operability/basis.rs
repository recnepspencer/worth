use crate::declaration::UiIntentOperabilityDependencyAxis;
use crate::runtime::expression::UiExpressionResultReference;

use super::{
    UiIntentConfirmationPosture, UiIntentMutabilityPosture, UiIntentOccupancyObservation,
    UiIntentOccupancyState, UiIntentPolicyPosture, UiIntentReadinessPosture,
    UiIntentSupportPosture,
};

mod axis_observation;

/// The decision axes one operability decision visits: support, mutability,
/// readiness, occupancy, policy, affinity and confirmation. Every decision
/// visits each axis exactly once, whatever source kind the axis reads, so the
/// count is the shape of the decision rather than a tally of sources.
const DECISION_AXES: usize = 7;

pub(crate) struct UiIntentOperabilityBasis {
    contract_identity: Box<str>,
    support: UiIntentSupportPosture,
    mutability: UiIntentMutabilityPosture,
    readiness: UiIntentReadinessPosture,
    occupancy: UiIntentOccupancyObservation,
    policy: UiIntentPolicyPosture,
    confirmation: UiIntentConfirmationPosture,
    query_inputs: Box<[worth_ui_query_binding::UiProjectionInputFactReference]>,
    application_inputs: Box<[super::super::payload::UiIntentApplicationInputReference]>,
    policy_input: Option<super::super::payload::UiIntentApplicationInputReference>,
    expression_inputs: Box<
        [(
            UiIntentOperabilityDependencyAxis,
            UiExpressionResultReference,
        )],
    >,
    confirmation_input: Option<super::super::payload::UiIntentApplicationInputReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentOperabilityDependencyDrift {
    DeclaredDependency,
    Policy,
    Confirmation,
    /// A condition result the decision read is no longer the retained result
    /// of the active generation.
    ExpressionResult {
        axis: UiIntentOperabilityDependencyAxis,
    },
}

pub(crate) fn observe_operability_basis(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    declaration: &crate::declaration::UiCanonicalIntentDeclaration,
    definition: &crate::capability::IntentDefinitionDescriptor,
    binding_support: crate::runtime::intent_execution::UiIntentExecutionBindingSupport,
    occupancy: &UiIntentOccupancyState,
) -> UiIntentOperabilityBasis {
    let mut inputs = axis_observation::UiIntentAxisInputs::default();
    let contract = declaration.operability();
    let mutability = axis_observation::observe_mutability(view, contract.mutability(), &mut inputs);
    let readiness = axis_observation::observe_readiness(view, contract.readiness(), &mut inputs);
    let policy = axis_observation::observe_policy(view, contract.policy(), &mut inputs);
    let confirmation = axis_observation::observe_confirmation(view, declaration.confirmation());
    UiIntentOperabilityBasis {
        contract_identity: contract.identity().into(),
        support: support(binding_support),
        mutability,
        readiness,
        occupancy: occupancy.observe(
            declaration.concurrency(),
            declaration,
            definition.id(),
            view.target(),
        ),
        policy: policy.posture,
        confirmation: confirmation.posture,
        query_inputs: inputs.query.into_boxed_slice(),
        application_inputs: inputs.application.into_boxed_slice(),
        policy_input: policy.input,
        expression_inputs: inputs.expressions.into_boxed_slice(),
        confirmation_input: confirmation.input,
    }
}

const fn support(
    binding: crate::runtime::intent_execution::UiIntentExecutionBindingSupport,
) -> UiIntentSupportPosture {
    match binding {
        crate::runtime::intent_execution::UiIntentExecutionBindingSupport::Supported => {
            UiIntentSupportPosture::Supported
        }
    }
}

impl UiIntentOperabilityBasis {
    pub(crate) fn decision(
        &self,
        affinity: super::UiIntentAffinityPosture,
    ) -> super::UiIntentOperabilityDecision {
        super::UiIntentOperabilityDecision::new(super::UiIntentOperabilityDecisionInput {
            contract_identity: self.contract_identity().into(),
            support: self.support(),
            mutability: self.mutability(),
            readiness: self.readiness(),
            occupancy: self.occupancy().posture(),
            policy: self.policy(),
            affinity,
            confirmation: self.confirmation(),
            selected_dependencies_visited: DECISION_AXES,
        })
    }

    pub(crate) fn contract_identity(&self) -> &str {
        &self.contract_identity
    }

    pub(crate) const fn support(&self) -> UiIntentSupportPosture {
        self.support
    }

    pub(crate) const fn mutability(&self) -> UiIntentMutabilityPosture {
        self.mutability
    }

    pub(crate) const fn readiness(&self) -> UiIntentReadinessPosture {
        self.readiness
    }

    pub(crate) const fn occupancy(&self) -> &UiIntentOccupancyObservation {
        &self.occupancy
    }

    pub(crate) const fn policy(&self) -> UiIntentPolicyPosture {
        self.policy
    }

    pub(crate) fn confirmation(&self) -> UiIntentConfirmationPosture {
        self.confirmation.clone()
    }

    pub(crate) fn retained_dependency_reference_count(&self) -> usize {
        self.query_inputs.len()
            + self.application_inputs.len()
            + usize::from(self.policy_input.is_some())
            + self.expression_inputs.len()
            + usize::from(self.confirmation_input.is_some())
    }

    pub(crate) fn currentness(
        &self,
        reads: &UiIntentOperabilityDependencyReads<'_>,
    ) -> Result<(), UiIntentOperabilityDependencyDrift> {
        let application_current = |expected| {
            reads
                .application_facts
                .is_current_reference(expected, reads.generation)
        };
        if self
            .policy_input
            .as_ref()
            .is_some_and(|expected| !application_current(expected))
        {
            return Err(UiIntentOperabilityDependencyDrift::Policy);
        }
        if self
            .confirmation_input
            .as_ref()
            .is_some_and(|expected| !application_current(expected))
        {
            return Err(UiIntentOperabilityDependencyDrift::Confirmation);
        }
        if let Some((axis, _)) = self.expression_inputs.iter().find(|(_, expected)| {
            !reads
                .expressions
                .is_current_result(expected, reads.generation)
        }) {
            return Err(UiIntentOperabilityDependencyDrift::ExpressionResult { axis: *axis });
        }
        let query_current = self.query_inputs.iter().all(|expected| {
            reads
                .mounted
                .current_projection_input(expected.revision().slot())
                .as_ref()
                == Some(expected)
        });
        if query_current && self.application_inputs.iter().all(application_current) {
            Ok(())
        } else {
            Err(UiIntentOperabilityDependencyDrift::DeclaredDependency)
        }
    }
}

/// The owners an operability basis is proven current against.
pub(crate) struct UiIntentOperabilityDependencyReads<'owners> {
    pub(crate) mounted: &'owners crate::mounting::WorthUiMountedSessionState,
    pub(crate) application_facts: &'owners super::super::payload::UiIntentApplicationFactState,
    pub(crate) expressions: &'owners crate::runtime::expression::UiExpressionRuntimeState,
    pub(crate) generation: &'owners crate::runtime::WorthUiActiveApplicationGenerationIdentity,
}
