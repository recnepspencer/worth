use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;

use super::effect_program::WorthQueryApplicationRealizedEffect;
use super::effect_validation::denial;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::authorization::{
    WorthQueryDelegationActivationBinding, WorthQueryDelegationActivationEffect,
};

/// Exact child activation produced from Query-owned delegation authority.
pub struct WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope> {
    program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub fn materialize_capability_delegation_program(
        self,
    ) -> Result<
        WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let binding = self
            .admission
            .delegation_activation_binding()
            .ok_or_else(|| transition_required(self.admission.operation()))?;
        let effects = activation_effects(binding, self.admission.allowed_graph_contract())?;
        let emission_retained_bytes_ceiling = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        let program = WorthQueryApplicationEffectProgram {
            read_set: self,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
        };
        validate_delegation_activation_program(&program)?;
        Ok(WorthQueryDelegationActivationProgram { program })
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>
{
    pub(super) fn into_inner(
        self,
    ) -> WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
        self.program
    }
}

pub(in crate::domain_computation::primary_graph) fn validate_delegation_activation_program<
    Schema,
    Operation,
    Input,
    Scope,
>(
    program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let binding = program
        .read_set
        .admission
        .delegation_activation_binding()
        .ok_or_else(|| transition_required(program.read_set.admission.operation()))?;
    let expected =
        activation_effects(binding, program.read_set.admission.allowed_graph_contract())?;
    if program.emission_retained_bytes == 0 && effects_are_exact(&program.effects, &expected) {
        Ok(())
    } else {
        Err(program_mismatch(program.read_set.admission.operation()))
    }
}

fn activation_effects(
    binding: &WorthQueryDelegationActivationBinding,
    installed: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
) -> Result<Vec<WorthQueryApplicationRealizedEffect>, WorthQueryApplicationAttemptDenial> {
    binding
        .materialize_program(installed)
        .map_err(|()| program_mismatch("delegation activation operation contract"))
        .map(|effects| effects.into_iter().map(lower_effect).collect())
}

fn lower_effect(
    effect: WorthQueryDelegationActivationEffect,
) -> WorthQueryApplicationRealizedEffect {
    match effect {
        WorthQueryDelegationActivationEffect::CreateEntity { kind, key, fields } => {
            WorthQueryApplicationRealizedEffect::CreateEntity { kind, key, fields }
        }
        WorthQueryDelegationActivationEffect::CreateRelation {
            kind,
            key,
            from,
            to,
        } => WorthQueryApplicationRealizedEffect::CreateRelation {
            kind,
            key,
            from,
            to,
        },
    }
}

fn effects_are_exact(
    actual: &[WorthQueryApplicationRealizedEffect],
    expected: &[WorthQueryApplicationRealizedEffect],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| effect_is_exact(actual, expected))
}

fn effect_is_exact(
    actual: &WorthQueryApplicationRealizedEffect,
    expected: &WorthQueryApplicationRealizedEffect,
) -> bool {
    match (actual, expected) {
        (
            WorthQueryApplicationRealizedEffect::CreateEntity { kind, key, fields },
            WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: expected_kind,
                key: expected_key,
                fields: expected_fields,
            },
        ) => kind == expected_kind && key == expected_key && fields == expected_fields,
        (
            WorthQueryApplicationRealizedEffect::CreateRelation {
                kind,
                key,
                from,
                to,
            },
            WorthQueryApplicationRealizedEffect::CreateRelation {
                kind: expected_kind,
                key: expected_key,
                from: expected_from,
                to: expected_to,
            },
        ) => {
            kind == expected_kind
                && key == expected_key
                && from == expected_from
                && to == expected_to
        }
        _ => false,
    }
}

fn transition_required(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::DelegationActivationRequired,
        subject,
    )
}

fn program_mismatch(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::DelegationActivationProgramMismatch,
        subject,
    )
}
