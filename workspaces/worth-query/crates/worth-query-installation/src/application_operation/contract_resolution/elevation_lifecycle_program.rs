use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityElevationRule, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityTransitionBinding,
    },
    application_schema::{
        ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
        ApplicationSchemaMember,
    },
};

pub(super) fn lifecycle_program_targets(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> Vec<ApplicationOperationProgramTarget> {
    members
        .iter()
        .find_map(|member| {
            let ApplicationSchemaMember::ApplicationCapability { contract } = member else {
                return None;
            };
            let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation()
            else {
                return None;
            };
            elevation
                .lifecycle()
                .transitions()
                .into_iter()
                .find_map(|transition| {
                    let installed = transition.operation();
                    if installed.operation() != operation || installed.input_type() != input_type {
                        return None;
                    }
                    let mut targets = Vec::new();
                    if transition == elevation.lifecycle().request() {
                        targets.extend(elevation.resource_relation().map(|relation| {
                            ApplicationOperationProgramTarget::Link {
                                relation: relation.relation().to_string(),
                                from: relation.from().to_string(),
                                to: relation.to().to_string(),
                            }
                        }));
                    }
                    targets.extend(transition.lifecycle_effect().map(|effect| {
                        ApplicationOperationProgramTarget::Emit {
                            effect: effect.effect().to_string(),
                        }
                    }));
                    Some(targets)
                })
        })
        .unwrap_or_default()
}

pub(super) fn lifecycle_resource_decision_read(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> Option<ApplicationOperationDecisionReadTarget> {
    let candidates = members
        .iter()
        .filter_map(|member| {
            let ApplicationSchemaMember::ApplicationCapability { contract } = member else {
                return None;
            };
            let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation()
            else {
                return None;
            };
            let resource_relation = elevation.resource_relation()?;
            Some((elevation.lifecycle(), resource_relation))
        })
        .flat_map(|(lifecycle, resource_relation)| {
            [
                lifecycle.approve(),
                lifecycle.revoke(),
                lifecycle.complete_review(),
            ]
            .map(|transition| (transition, resource_relation))
        });
    exact_lifecycle_resource_decision_read(candidates, operation, input_type)
}

fn exact_lifecycle_resource_decision_read<'a>(
    candidates: impl IntoIterator<
        Item = (
            &'a ApplicationCapabilityTransitionBinding,
            &'a ApplicationCapabilityRelationBinding,
        ),
    >,
    operation: &str,
    input_type: &str,
) -> Option<ApplicationOperationDecisionReadTarget> {
    candidates
        .into_iter()
        .find(|(transition, _)| {
            transition.operation().operation() == operation
                && transition.operation().input_type() == input_type
        })
        .map(
            |(_, relation)| ApplicationOperationDecisionReadTarget::Relation {
                relation: relation.relation().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::{
        application_capability::{
            ApplicationCapabilityLifecycleEffect, ApplicationCapabilityMarkerIdentity,
            ApplicationCapabilityRef, ApplicationCapabilityRelationBinding,
            ApplicationCapabilityTransitionBinding,
        },
        application_schema::{
            ApplicationEffectMarkerIdentity, ApplicationEffectRef,
            ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationRelationRef,
            ApplicationRetainedEffectBinding, ApplicationStructuredValueBinding, OperationEmits,
        },
    };

    struct Schema;
    struct Operation;
    struct Effect;
    struct FirstInput;
    struct SecondInput;
    struct FirstRelation;
    struct SecondRelation;
    struct Elevation;
    struct FirstResource;
    struct SecondResource;

    worth_query_declaration::worth_query_capability!(
        Capability in Schema,
        identity "worth.query.installation-test.lifecycle-capability.v1"
    );
    worth_query_declaration::worth_query_capability!(
        FirstCapability in Schema,
        identity "worth.query.installation-test.first-lifecycle-capability.v1"
    );
    worth_query_declaration::worth_query_capability!(
        SecondCapability in Schema,
        identity "worth.query.installation-test.second-lifecycle-capability.v1"
    );

    worth_query_declaration::worth_query_portable_type!(
        FirstInput => "worth.query.installation-test.first-lifecycle-input"
    );
    worth_query_declaration::worth_query_portable_type!(
        SecondInput => "worth.query.installation-test.second-lifecycle-input"
    );
    worth_query_declaration::worth_query_structured_value_binding!(
        LifecycleOperationInputBinding for String { identity: "worth.rust.string" }
    );
    worth_query_declaration::worth_query_structured_value_binding!(
        LifecycleEffectPayloadBinding for String { identity: "worth.rust.string" }
    );
    worth_query_declaration::worth_query_structured_value_binding!(
        FirstInputBinding for FirstInput { identity: "worth.query.installation-test.first-lifecycle-input" }
    );
    worth_query_declaration::worth_query_structured_value_binding!(
        SecondInputBinding for SecondInput { identity: "worth.query.installation-test.second-lifecycle-input" }
    );

    impl ApplicationRetainedEffectBinding for LifecycleEffectPayloadBinding {
        fn retained_bytes(value: &Self::Value) -> u64 {
            std::mem::size_of::<String>() as u64 + value.capacity() as u64
        }
    }

    impl ApplicationOperationMarkerIdentity<Schema> for Operation {
        type InputBinding = LifecycleOperationInputBinding;
        const IDENTIFIER: &'static str = "Run";
    }

    impl ApplicationEffectMarkerIdentity<Schema> for Effect {
        type PayloadBinding = LifecycleEffectPayloadBinding;
        const IDENTIFIER: &'static str = "ActivityEffect";
    }

    impl OperationEmits<Operation> for Effect {}

    impl ApplicationCapabilityLifecycleEffect<Schema, Operation> for String {
        type Effect = Effect;
        type PayloadBinding = LifecycleEffectPayloadBinding;

        fn effect() -> ApplicationEffectRef<Schema, Self::Effect, String> {
            ApplicationEffectRef::from_declaration()
        }

        fn lifecycle_effect(&self) -> Option<String> {
            Some(self.clone())
        }
    }

    #[test]
    fn installed_target_comes_from_the_typed_transition_binding() {
        let transition =
            ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
                ApplicationOperationRef::<Schema, Operation, String>::from_declaration(),
            );
        let target = transition.lifecycle_effect().map(|effect| {
            super::ApplicationOperationProgramTarget::Emit {
                effect: effect.effect().to_owned(),
            }
        });
        assert_eq!(
            target,
            Some(super::ApplicationOperationProgramTarget::Emit {
                effect: "ActivityEffect".to_owned(),
            })
        );
    }

    #[test]
    fn same_named_lifecycle_operations_select_resource_read_by_exact_input_type() {
        let first_transition = transition::<FirstCapability, FirstInputBinding>();
        let second_transition = transition::<SecondCapability, SecondInputBinding>();
        let first_relation = resource_relation::<FirstRelation, FirstResource>("FirstResource");
        let second_relation = resource_relation::<SecondRelation, SecondResource>("SecondResource");

        let selected = super::exact_lifecycle_resource_decision_read(
            [
                (&first_transition, &first_relation),
                (&second_transition, &second_relation),
            ],
            "Advance",
            SecondInputBinding::IDENTITY.as_str(),
        );

        assert_eq!(
            selected,
            Some(super::ApplicationOperationDecisionReadTarget::Relation {
                relation: "SecondResource".to_owned(),
                from: "Elevation".to_owned(),
                to: "SecondResource".to_owned(),
            })
        );
    }

    fn transition<Capability, InputBinding>() -> ApplicationCapabilityTransitionBinding
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema>,
        InputBinding: ApplicationStructuredValueBinding,
    {
        struct TransitionOperation<InputBinding>(std::marker::PhantomData<InputBinding>);
        impl<InputBinding> ApplicationOperationMarkerIdentity<Schema> for TransitionOperation<InputBinding>
        where
            InputBinding: ApplicationStructuredValueBinding,
        {
            type InputBinding = InputBinding;
            const IDENTIFIER: &'static str = "Advance";
        }
        ApplicationCapabilityTransitionBinding::from_references(
            ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
            ApplicationOperationRef::<
                Schema,
                TransitionOperation<InputBinding>,
                InputBinding::Value,
            >::from_declaration(),
        )
    }

    fn resource_relation<Relation, Resource>(
        name: &'static str,
    ) -> ApplicationCapabilityRelationBinding {
        ApplicationCapabilityRelationBinding::from_reference(ApplicationRelationRef::<
            Schema,
            Relation,
            Elevation,
            Resource,
        >::from_schema_identifiers(
            name, "Elevation", name,
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ))
    }
}
