use std::collections::BTreeMap;

use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::effect_program::WorthQueryApplicationRealizedEffect;
use super::effect_validation::{canonical_key, denial};
use super::elevation_lifecycle_emission::{append_lifecycle_emission, lifecycle_emission_is_exact};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::application_contract_admission::application_contract_exactly_matches_program_targets;
use crate::domain_computation::authorization::WorthQueryElevationRequestBinding;

/// Exact request mutation produced from Query-owned lifecycle authority.
///
/// Callers cannot construct this wrapper or supply requester, status, or time
/// facts. They may only obtain it after the ordinary invariant read phase has
/// completed for an admitted elevation request.
pub struct WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope> {
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
    pub fn materialize_elevation_request_program(
        self,
    ) -> Result<
        WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let binding = self
            .admission
            .elevation_request_binding()
            .ok_or_else(|| transition_required(self.admission.operation()))?;
        validate_installed_program(binding, self.admission.allowed_graph_contract())?;
        let mut effects = request_effects(binding)?;
        let emission_retained_bytes_ceiling = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        let emission_retained_bytes = append_lifecycle_emission(
            &mut effects,
            binding.lifecycle_effect(),
            emission_retained_bytes_ceiling,
            self.admission.operation(),
        )?;
        let program = WorthQueryApplicationEffectProgram {
            read_set: self,
            effects,
            emission_retained_bytes,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            validator_work_admission:
                super::effect_program::WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
            output_correspondence: Default::default(),
        };
        validate_elevation_request_program(&program)?;
        Ok(WorthQueryElevationRequestProgram { program })
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>
{
    pub(super) fn into_inner(
        self,
    ) -> WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
        self.program
    }
}

pub(in crate::domain_computation::primary_graph) fn validate_elevation_request_program<
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
        .elevation_request_binding()
        .ok_or_else(|| transition_required(program.read_set.admission.operation()))?;
    let expected = request_effects(binding)?;
    let base_count = 6 + usize::from(binding.resource_relation().is_some());
    if expected.len() == base_count
        && program
            .effects
            .get(..base_count)
            .is_some_and(|actual| same_effects(actual, &expected))
        && lifecycle_emission_is_exact(
            &program.effects,
            base_count,
            binding.lifecycle_effect(),
            program.emission_retained_bytes,
        )
    {
        Ok(())
    } else {
        Err(program_mismatch(program.read_set.admission.operation()))
    }
}

fn validate_installed_program(
    binding: &WorthQueryElevationRequestBinding,
    installed: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if application_contract_exactly_matches_program_targets(
        installed,
        binding.required_program_targets(),
    ) {
        Ok(())
    } else {
        Err(program_mismatch("elevation request operation program"))
    }
}

fn request_effects(
    binding: &WorthQueryElevationRequestBinding,
) -> Result<Vec<WorthQueryApplicationRealizedEffect>, WorthQueryApplicationAttemptDenial> {
    let elevation_key = canonical_key(binding.elevation_key().to_owned(), "elevation request")?;
    let review_key = canonical_key(binding.review_key().to_owned(), "mandatory review")?;
    let elevation = created(binding.elevation_kind(), &elevation_key);
    let review = created(binding.review_kind(), &review_key);
    let elevation_fields = elevation_fields(binding)?;
    let review_fields = review_fields(binding)?;
    let mut effects = vec![
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: binding.elevation_kind(),
            key: elevation_key.clone(),
            fields: elevation_fields,
        },
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: binding.review_kind(),
            key: review_key.clone(),
            fields: review_fields,
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: binding.requester_relation(),
            key: elevation_key.clone(),
            from: EntityReference::Existing(binding.requester()),
            to: elevation.clone(),
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: binding.grant_relation(),
            key: elevation_key.clone(),
            from: elevation.clone(),
            to: EntityReference::Existing(binding.grant()),
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: binding.review_relation(),
            key: elevation_key.clone(),
            from: elevation.clone(),
            to: review.clone(),
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: binding.review_scope_relation(),
            key: review_key,
            from: review,
            to: EntityReference::Existing(binding.resource()),
        },
    ];
    if let Some(resource_relation) = binding.resource_relation() {
        effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: resource_relation,
            key: elevation_key,
            from: elevation,
            to: EntityReference::Existing(binding.resource()),
        });
    }
    Ok(effects)
}

fn elevation_fields(
    binding: &WorthQueryElevationRequestBinding,
) -> Result<
    BTreeMap<
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    >,
    WorthQueryApplicationAttemptDenial,
> {
    exact_fields(
        [
            (
                binding.elevation_identity_field(),
                binding.elevation_identity(),
            ),
            (binding.reason_field(), binding.reason()),
            (binding.status_field(), binding.requested_status()),
            (binding.not_before_field(), binding.issued_at()),
            (binding.not_after_field(), binding.expires_at()),
        ],
        5,
        "elevation request fields",
    )
}

fn review_fields(
    binding: &WorthQueryElevationRequestBinding,
) -> Result<
    BTreeMap<
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    >,
    WorthQueryApplicationAttemptDenial,
> {
    exact_fields(
        [
            (binding.review_identity_field(), binding.review_identity()),
            (binding.review_type_field(), binding.review_type()),
            (
                binding.review_status_field(),
                binding.review_required_status(),
            ),
        ],
        3,
        "mandatory review fields",
    )
}

fn exact_fields<'a, const N: usize>(
    fields: [(
        &'a worth_foundational::facade::AspectFieldLocator,
        &'a worth_foundational::facade::AspectValue,
    ); N],
    expected_len: usize,
    subject: &str,
) -> Result<
    BTreeMap<
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    >,
    WorthQueryApplicationAttemptDenial,
> {
    let fields = fields
        .into_iter()
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    if fields.len() == expected_len {
        Ok(fields)
    } else {
        Err(program_mismatch(subject))
    }
}

fn created(kind_id: worth_relational::facade::identity::KindId, key: &str) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: ClientKey::raw(key.to_string()),
    })
}

fn same_effects(
    actual: &[WorthQueryApplicationRealizedEffect],
    expected: &[WorthQueryApplicationRealizedEffect],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| same_effect(actual, expected))
}

fn same_effect(
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
        WorthQueryApplicationAttemptDenialKind::ElevationTransitionRequired,
        subject,
    )
}

fn program_mismatch(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ElevationRequestProgramMismatch,
        subject,
    )
}
