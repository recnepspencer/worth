use worth_query_declaration::facade::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationExternalEffectBinding,
    ApplicationRetainedEffectBinding, ApplicationStructuredValueBinding, OperationEmits,
};
use worth_query_installation::facade::ApplicationOperationProgramTarget;

use super::{
    denial, CandidateItemKind, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationEmission, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// An installed effect declaration is not enough; the operation itself
    /// must carry the compile-time emit capability:
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::application_schema::ApplicationEffectRef;
    /// use worth_query_execution::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder;
    ///
    /// struct Schema;
    /// struct Operation;
    /// struct Input;
    /// struct Scope;
    /// worth_query_declaration::worth_query_structured_value_binding!(
    ///     UndeclaredEffectPayloadBinding for String {
    ///         identity: "worth.example.undeclared-effect-payload.v1"
    ///     }
    /// );
    /// worth_query_declaration::worth_query_effect!(
    ///     UndeclaredEffect for Schema,
    ///     payload UndeclaredEffectPayloadBinding
    /// );
    ///
    /// fn cannot_emit_undeclared_effect(
    ///     builder: &mut WorthQueryApplicationEffectProgramBuilder<
    ///         Schema, Operation, Input, Scope,
    ///     >,
    /// ) {
    ///     let effect = UndeclaredEffect::reference();
    ///     builder.emit(effect, "payload".to_owned()).unwrap();
    /// }
    /// ```
    pub fn emit<Effect, Payload>(
        &mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        payload: Payload,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation>,
        Effect::PayloadBinding: ApplicationRetainedEffectBinding<Value = Payload>,
        Payload: Send + Sync + 'static,
    {
        self.admit_program_target(&ApplicationOperationProgramTarget::Emit {
            effect: effect.name().to_string(),
        })?;
        Effect::PayloadBinding::validate(&payload)
            .map_err(|_| invalid_payload_denial(effect.name()))?;
        let payload_retained_bytes = Effect::PayloadBinding::retained_bytes(&payload);
        let Some(retained_bytes) = self
            .emission_retained_bytes
            .checked_add(payload_retained_bytes)
        else {
            return Err(retained_bytes_denial(effect.name()));
        };
        if retained_bytes > self.emission_retained_bytes_ceiling {
            return Err(retained_bytes_denial(effect.name()));
        }
        let emission =
            WorthQueryApplicationEmission::new::<Effect::PayloadBinding>(effect.name(), payload);
        let candidate_retained_representation_bytes = emission
            .candidate_retained_representation_bytes()
            .ok_or_else(|| candidate_bytes_denial(effect.name()))?;
        self.charge_candidate_representation(
            CandidateItemKind::Emit,
            candidate_retained_representation_bytes,
            0,
        )?;
        self.effects
            .push(WorthQueryApplicationRealizedEffect::Emit(emission));
        self.emission_retained_bytes = retained_bytes;
        Ok(())
    }

    /// Emits the exact typed payload selected by an installed external-effect
    /// contract. The provider later matches this emission to that contract;
    /// callers never supply wire bytes at commit or dispatch.
    pub fn emit_external<Effect, Payload>(
        &mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        payload: Payload,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation>,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<Value = Payload>,
        Payload: Send + Sync + 'static,
    {
        self.admit_program_target(&ApplicationOperationProgramTarget::Emit {
            effect: effect.name().to_string(),
        })?;
        Effect::PayloadBinding::validate(&payload)
            .map_err(|_| invalid_payload_denial(effect.name()))?;
        let payload_retained_bytes = Effect::PayloadBinding::retained_bytes(&payload);
        let Some(retained_bytes) = self
            .emission_retained_bytes
            .checked_add(payload_retained_bytes)
        else {
            return Err(retained_bytes_denial(effect.name()));
        };
        if retained_bytes > self.emission_retained_bytes_ceiling {
            return Err(retained_bytes_denial(effect.name()));
        }
        let emission = WorthQueryApplicationEmission::new_external::<Effect::PayloadBinding>(
            effect.name(),
            payload,
        )
        .map_err(|()| external_payload_denial(effect.name()))?;
        let candidate_retained_representation_bytes = emission
            .candidate_retained_representation_bytes()
            .ok_or_else(|| candidate_bytes_denial(effect.name()))?;
        self.charge_candidate_representation(
            CandidateItemKind::Emit,
            candidate_retained_representation_bytes,
            0,
        )?;
        self.effects
            .push(WorthQueryApplicationRealizedEffect::Emit(emission));
        self.emission_retained_bytes = retained_bytes;
        Ok(())
    }
}

fn retained_bytes_denial(effect: &str) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded,
        effect,
    )
}

fn candidate_bytes_denial(effect: &str) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
        effect,
    )
}

fn invalid_payload_denial(effect: &str) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
        effect,
    )
}

fn external_payload_denial(effect: &str) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ExternalEffectPayloadProjectionRejected,
        effect,
    )
}
