use worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect;

use super::effect_program::{WorthQueryApplicationEmission, WorthQueryApplicationRealizedEffect};
use super::effect_validation::denial;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

pub(super) fn append_lifecycle_emission(
    effects: &mut Vec<WorthQueryApplicationRealizedEffect>,
    derived: Option<&DerivedApplicationCapabilityLifecycleEffect>,
    retained_bytes_ceiling: u64,
    subject: &str,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let Some(derived) = derived else {
        return Ok(0);
    };
    if derived.retained_bytes() > retained_bytes_ceiling {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded,
            subject,
        ));
    }
    effects.push(WorthQueryApplicationRealizedEffect::Emit(
        WorthQueryApplicationEmission::from_lifecycle(derived),
    ));
    Ok(derived.retained_bytes())
}

pub(super) fn lifecycle_emission_is_exact(
    effects: &[WorthQueryApplicationRealizedEffect],
    base_effect_count: usize,
    derived: Option<&DerivedApplicationCapabilityLifecycleEffect>,
    retained_bytes: u64,
) -> bool {
    match derived {
        None => effects.len() == base_effect_count && retained_bytes == 0,
        Some(derived) => {
            effects.len() == base_effect_count.saturating_add(1)
                && retained_bytes == derived.retained_bytes()
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
            ApplicationOperationMarkerIdentity, ApplicationOperationRef,
            ApplicationRetainedEffectBinding, OperationEmits,
        },
    };

    use super::*;

    pub struct Schema;
    struct Operation;
    pub struct Effect;
    worth_query_declaration::worth_query_structured_value_binding!(InputBinding for String { identity: "worth.rust.string" });
    worth_query_declaration::worth_query_structured_value_binding!(pub PayloadBinding for String { identity: "worth.rust.string" });
    impl ApplicationRetainedEffectBinding for PayloadBinding {
        fn retained_bytes(value: &Self::Value) -> u64 {
            u64::try_from(std::mem::size_of::<Self::Value>())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(value.capacity()).unwrap_or(u64::MAX))
        }
    }

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
        let retained =
            append_lifecycle_emission(&mut effects, Some(&derived), u64::MAX, "Run").unwrap();
        assert!(lifecycle_emission_is_exact(
            &effects,
            0,
            Some(&derived),
            retained,
        ));

        let omitted = Vec::new();
        assert!(!lifecycle_emission_is_exact(
            &omitted,
            0,
            Some(&derived),
            retained,
        ));

        effects.push(WorthQueryApplicationRealizedEffect::Emit(
            WorthQueryApplicationEmission::new::<PayloadBinding>("ExtraEffect", "extra".to_owned()),
        ));
        assert!(!lifecycle_emission_is_exact(
            &effects,
            0,
            Some(&derived),
            retained,
        ));

        let retargeted = vec![WorthQueryApplicationRealizedEffect::Emit(
            WorthQueryApplicationEmission::new::<PayloadBinding>("OtherEffect", input),
        )];
        assert!(!lifecycle_emission_is_exact(
            &retargeted,
            0,
            Some(&derived),
            retained,
        ));
    }
}
