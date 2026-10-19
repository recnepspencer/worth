//! Only an earned native body selection can suppress a mandatory external slot.
use super::*;
use crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn body_external_emission(
        &mut self,
        mut selection: BodyExternalEffectSelection,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let target = ApplicationOperationProgramTarget::Emit {
            effect: selection.meaning().effect.clone(),
        };
        self.admit_program_target(&target)?;
        let declared = self
            .read_set
            .admission
            .allowed_graph_contract()
            .body()
            .and_then(|body| body.external_emission());
        if self.read_set.body_external_emission.is_some()
            || !declared.is_some_and(|meaning| selection.matches(meaning))
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                selection.recipe().operation(),
            ));
        }
        if selection.emits() {
            let payload = selection.take_payload().ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::ExternalEffectPayloadProjectionRejected,
                    &selection.meaning().effect,
                )
            })?;
            let emission = WorthQueryApplicationEmission::from_external_projection(
                payload,
                selection.funding(),
                selection.backing(),
            );
            let retained = self
                .emission_retained_bytes
                .checked_add(emission.retained_bytes())
                .filter(|retained| *retained <= self.emission_retained_bytes_ceiling)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded,
                        &selection.meaning().effect,
                    )
                })?;
            let bytes = emission
                .candidate_retained_representation_bytes()
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        &selection.meaning().effect,
                    )
                })?;
            self.charge_candidate_representation(CandidateItemKind::Emit, bytes, 0)?;
            self.effects
                .push(WorthQueryApplicationRealizedEffect::Emit(emission));
            self.emission_retained_bytes = retained;
        }
        self.read_set.body_external_emission = Some(selection);
        Ok(())
    }
    pub(super) fn validate_body_external_emission(
        &self,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let expected = self
            .read_set
            .admission
            .allowed_graph_contract()
            .body()
            .and_then(|body| body.external_emission());
        match (expected, &self.read_set.body_external_emission) {
            (None, None) => Ok(()),
            (Some(expected), Some(selected)) if selected.matches(expected) => {
                let count = self.effects.iter().filter(|effect| matches!(effect, WorthQueryApplicationRealizedEffect::Emit(emission) if emission.effect_name() == expected.effect)).count();
                if count == usize::from(selected.emits()) {
                    Ok(())
                } else {
                    Err(denial(
                        WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                        &expected.effect,
                    ))
                }
            }
            _ => Err(denial(
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                self.read_set.admission.operation(),
            )),
        }
    }
}
