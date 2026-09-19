use crate::declaration::{
    UiResolvedIntentConfirmationSource, UiResolvedIntentMutabilitySource,
    UiResolvedIntentReadinessSource,
};

use super::{
    UiIntentConfirmationPosture, UiIntentMutabilityPosture, UiIntentOccupancyObservation,
    UiIntentOccupancyState, UiIntentPolicyPosture, UiIntentReadinessPosture,
    UiIntentSupportPosture,
};

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
    policy_input: super::super::payload::UiIntentApplicationInputReference,
    confirmation_input: Option<super::super::payload::UiIntentApplicationInputReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentOperabilityDependencyDrift {
    DeclaredDependency,
    Policy,
    Confirmation,
}

pub(crate) fn observe_operability_basis(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    declaration: &crate::declaration::UiCanonicalIntentDeclaration,
    definition: &crate::capability::IntentDefinitionDescriptor,
    binding_support: crate::runtime::intent_execution::UiIntentExecutionBindingSupport,
    occupancy: &UiIntentOccupancyState,
) -> UiIntentOperabilityBasis {
    let mut query_inputs = Vec::new();
    let mut application_inputs = Vec::new();
    let mutability = observe_mutability(
        view,
        declaration.operability().mutability(),
        &mut query_inputs,
        &mut application_inputs,
    );
    let readiness = observe_readiness(
        view,
        declaration.operability().readiness(),
        &mut query_inputs,
        &mut application_inputs,
    );
    let policy_input =
        application_boolean_reference(view, declaration.operability().policy().slot());
    let confirmation = observe_confirmation(view, declaration.confirmation());
    UiIntentOperabilityBasis {
        contract_identity: declaration.operability().identity().into(),
        support: support(binding_support),
        mutability,
        readiness,
        occupancy: occupancy.observe(
            declaration.concurrency(),
            declaration,
            definition.id(),
            view.target(),
        ),
        policy: boolean_policy(
            policy_input
                .boolean_value()
                .expect("resolved policy input has Boolean shape"),
        ),
        confirmation: confirmation.posture,
        query_inputs: query_inputs.into_boxed_slice(),
        application_inputs: application_inputs.into_boxed_slice(),
        policy_input,
        confirmation_input: confirmation.input,
    }
}

fn observe_mutability(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    source: &UiResolvedIntentMutabilitySource,
    query_inputs: &mut Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    application_inputs: &mut Vec<super::super::payload::UiIntentApplicationInputReference>,
) -> UiIntentMutabilityPosture {
    match source {
        UiResolvedIntentMutabilitySource::ApplicationBoolean(slot) => {
            if application_boolean(view, *slot, application_inputs) {
                UiIntentMutabilityPosture::Writable
            } else {
                UiIntentMutabilityPosture::Readonly
            }
        }
        UiResolvedIntentMutabilitySource::ProjectionReadonly { identity, slot } => {
            retain_projection(view, identity, *slot, query_inputs);
            UiIntentMutabilityPosture::Readonly
        }
        UiResolvedIntentMutabilitySource::CommittedDraft => UiIntentMutabilityPosture::Writable,
    }
}

fn observe_readiness(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    source: &UiResolvedIntentReadinessSource,
    query_inputs: &mut Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    application_inputs: &mut Vec<super::super::payload::UiIntentApplicationInputReference>,
) -> UiIntentReadinessPosture {
    match source {
        UiResolvedIntentReadinessSource::ApplicationBoolean(slot) => {
            if application_boolean(view, *slot, application_inputs) {
                UiIntentReadinessPosture::Ready
            } else {
                UiIntentReadinessPosture::Pending
            }
        }
        UiResolvedIntentReadinessSource::Projection { identity, slot } => {
            if retain_projection(view, identity, *slot, query_inputs) {
                UiIntentReadinessPosture::Ready
            } else {
                UiIntentReadinessPosture::Pending
            }
        }
        UiResolvedIntentReadinessSource::CommittedDraft => UiIntentReadinessPosture::Ready,
    }
}

struct ObservedConfirmation {
    posture: UiIntentConfirmationPosture,
    input: Option<super::super::payload::UiIntentApplicationInputReference>,
}

fn observe_confirmation(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    contract: &crate::declaration::UiResolvedIntentConfirmationContract,
) -> ObservedConfirmation {
    match contract.source() {
        UiResolvedIntentConfirmationSource::NotRequired => ObservedConfirmation {
            posture: UiIntentConfirmationPosture::NotRequired,
            input: None,
        },
        UiResolvedIntentConfirmationSource::ApplicationBoolean(slot) => {
            let input = application_boolean_reference(view, *slot);
            let posture = if input
                .boolean_value()
                .expect("resolved confirmation input has Boolean shape")
            {
                UiIntentConfirmationPosture::Required {
                    policy_identity: contract.policy_identity().into(),
                }
            } else {
                UiIntentConfirmationPosture::NotRequired
            };
            ObservedConfirmation {
                posture,
                input: Some(input),
            }
        }
    }
}

fn application_boolean(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    slot: crate::declaration::UiIntentApplicationFactSlot,
    retained: &mut Vec<super::super::payload::UiIntentApplicationInputReference>,
) -> bool {
    let input = view
        .application(slot)
        .expect("resolved application fact slot exists in the active fact state");
    assert_eq!(
        input.revision().generation(),
        view.generation(),
        "operability fact must share the payload generation"
    );
    let value = input
        .boolean_value()
        .expect("resolved operability fact has Boolean shape");
    retained.push(input);
    value
}

fn application_boolean_reference(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    slot: crate::declaration::UiIntentApplicationFactSlot,
) -> super::super::payload::UiIntentApplicationInputReference {
    let input = view
        .application(slot)
        .expect("resolved application fact slot exists in the active fact state");
    assert_eq!(
        input.revision().generation(),
        view.generation(),
        "operability fact must share the payload generation"
    );
    input
}

fn retain_projection(
    view: &super::super::payload::UiIntentInputBasisView<'_>,
    expected: &worth_ui_query_binding::WorthUiQueryViewIdentity,
    slot: worth_ui_query_binding::UiProjectionInputSlot,
    retained: &mut Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
) -> bool {
    let Some(input) = view.projection(slot) else {
        return false;
    };
    let current = input.revision().projection_identity() == expected
        && input.revision().slot() == slot
        && input.posture() == worth_ui_query_binding::UiProjectionInputPosture::Current;
    retained.push(input);
    current
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

const fn boolean_policy(admitted: bool) -> UiIntentPolicyPosture {
    if admitted {
        UiIntentPolicyPosture::Admitted
    } else {
        UiIntentPolicyPosture::Denied
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
            selected_dependencies_visited: 7,
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
            + 1
            + usize::from(self.confirmation_input.is_some())
    }

    pub(crate) fn currentness(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        application_facts: &super::super::payload::UiIntentApplicationFactState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<(), UiIntentOperabilityDependencyDrift> {
        if !application_facts.is_current_reference(&self.policy_input, generation) {
            return Err(UiIntentOperabilityDependencyDrift::Policy);
        }
        if self
            .confirmation_input
            .as_ref()
            .is_some_and(|expected| !application_facts.is_current_reference(expected, generation))
        {
            return Err(UiIntentOperabilityDependencyDrift::Confirmation);
        }
        let query_current = self.query_inputs.iter().all(|expected| {
            mounted
                .current_projection_input(expected.revision().slot())
                .as_ref()
                == Some(expected)
        });
        let application_current = self
            .application_inputs
            .iter()
            .all(|expected| application_facts.is_current_reference(expected, generation));
        if query_current && application_current {
            Ok(())
        } else {
            Err(UiIntentOperabilityDependencyDrift::DeclaredDependency)
        }
    }
}
