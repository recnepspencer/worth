use worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect;

use super::effect_program::{WorthQueryApplicationEmission, WorthQueryApplicationRealizedEffect};
use super::effect_validation::denial;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

pub(super) fn append_lifecycle_emission(
    effects: &mut Vec<WorthQueryApplicationRealizedEffect>,
    derived: Option<&DerivedApplicationCapabilityLifecycleEffect>,
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let Some(derived) = derived else {
        return Ok(());
    };
    effects.try_reserve(1).map_err(|_| {
        denial(
            WorthQueryApplicationAttemptDenialKind::AllocationDenied,
            subject,
        )
    })?;
    effects.push(WorthQueryApplicationRealizedEffect::Emit(
        WorthQueryApplicationEmission::from_lifecycle(derived),
    ));
    Ok(())
}

pub(super) fn lifecycle_emission_is_exact(
    effects: &[WorthQueryApplicationRealizedEffect],
    base_effect_count: usize,
    derived: Option<&DerivedApplicationCapabilityLifecycleEffect>,
) -> bool {
    match derived {
        None => effects.len() == base_effect_count,
        Some(derived) => {
            effects.len() == base_effect_count.saturating_add(1)
                && matches!(
                    effects.last(),
                    Some(WorthQueryApplicationRealizedEffect::Emit(emission))
                        if emission.is_exact_lifecycle(derived)
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::{
        application_capability::{
            ApplicationCapabilityLifecycleEffect, ApplicationCapabilityRef,
            ApplicationCapabilityTransitionBinding,
        },
        application_schema::{
            ApplicationEffectMarkerIdentity, ApplicationEffectRef,
            ApplicationOperationMarkerIdentity, ApplicationOperationRef, OperationEmits,
        },
    };

    use super::*;

    pub struct Schema;
    struct Operation;
    pub struct Effect;
    worth_query_declaration::worth_query_structured_value_binding!(InputBinding for String { identity: "worth.rust.string" });
    worth_query_declaration::worth_query_structured_value_binding!(pub PayloadBinding for String { identity: "worth.rust.string" });

    worth_query_declaration::worth_query_capability!(
        Capability in Schema,
        identity "worth.query.execution-test.lifecycle-capability.v1"
    );

    impl ApplicationOperationMarkerIdentity<Schema> for Operation {
        type InputBinding = InputBinding;
        const IDENTIFIER: &'static str = "Run";
    }

    impl ApplicationEffectMarkerIdentity<Schema> for Effect {
        type PayloadBinding = PayloadBinding;
        const IDENTIFIER: &'static str = "ActivityEffect";
    }

    impl OperationEmits<Operation> for Effect {}

    impl ApplicationCapabilityLifecycleEffect<Schema, Operation> for String {
        type Effect = Effect;
        type PayloadBinding = PayloadBinding;

        fn effect() -> ApplicationEffectRef<Schema, Self::Effect, String> {
            ApplicationEffectRef::from_declaration()
        }

        fn lifecycle_effect(&self) -> Option<String> {
            Some(self.clone())
        }
    }

    #[test]
    fn omitted_extra_and_retargeted_lifecycle_emissions_are_not_exact() {
        let transition =
            ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
                ApplicationOperationRef::<Schema, Operation, String>::from_declaration(),
            );
        let binding = transition.lifecycle_effect().unwrap();
        let input = "estate:access".to_owned();
        let derived = worth_query_declaration::lifecycle_effect_derivation_authority::derive_application_capability_lifecycle_effect(
            binding,
            &input as &dyn std::any::Any,
        )
        .unwrap();
        let mut effects = Vec::new();
        append_lifecycle_emission(&mut effects, Some(&derived), "Run").unwrap();
        assert!(lifecycle_emission_is_exact(&effects, 0, Some(&derived),));

        let omitted = Vec::new();
        assert!(!lifecycle_emission_is_exact(&omitted, 0, Some(&derived),));

        effects.push(WorthQueryApplicationRealizedEffect::Emit(
            WorthQueryApplicationEmission::new::<PayloadBinding>("ExtraEffect", "extra".to_owned()),
        ));
        assert!(!lifecycle_emission_is_exact(&effects, 0, Some(&derived),));

        let retargeted = vec![WorthQueryApplicationRealizedEffect::Emit(
            WorthQueryApplicationEmission::new::<PayloadBinding>("OtherEffect", input),
        )];
        assert!(!lifecycle_emission_is_exact(&retargeted, 0, Some(&derived),));
    }
}
