use worth_runtime_world::facade::ProductBranchObservation;

use super::effect_program::WorthQueryApplicationRealizedEffect;
use super::provider_binding::prepare_program_migration_batch;
use super::WorthQueryApplicationEffectProgram;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProgramMigrationPreparationDenial,
};

pub(in crate::domain_computation::primary_graph) fn prepare_program_migration_effects<
    Schema,
    Operation,
    Input,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<
    (
        ProductBranchObservation,
        worth_relational::facade::transactions::WorkerIntentBatch,
        usize,
    ),
    WorthQueryProgramMigrationPreparationDenial,
>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    if !program.belongs_to_application(
        application.runtime.authority_identity(),
        &application.installed_schema.binding_identity(),
    ) {
        return Err(WorthQueryProgramMigrationPreparationDenial::ForeignCandidate);
    }
    let WorthQueryApplicationEffectProgram {
        read_set,
        effects,
        emission_retained_bytes,
        emission_retained_bytes_ceiling: _,
        conditional_definition,
        platform_mutation,
        // Sealing consumes the candidate reservation here; adoption admits and
        // accounts its complete target-rule validation as transaction work.
        validator_work_admission: _candidate_validator_work_admission,
        output_correspondence,
        retain_output_demand_observation,
        retain_client_observation,
        producer_required_invariants,
        output_currentness_facts,
    } = program;

    let contracts = read_set.admission.allowed_graph_contract();
    if contracts.external_effect().is_declared() {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot perform an external effect",
            ),
        );
    }
    if contracts.aftermath().is_some() {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot own an aftermath contract",
            ),
        );
    }
    if conditional_definition.is_some() {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot advance a conditional definition",
            ),
        );
    }
    if platform_mutation {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot author platform effects",
            ),
        );
    }
    if emission_retained_bytes != 0
        || effects
            .iter()
            .any(|effect| matches!(effect, WorthQueryApplicationRealizedEffect::Emit(_)))
    {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot emit application output",
            ),
        );
    }
    if !output_correspondence.is_empty()
        || retain_output_demand_observation
        || retain_client_observation
        || !producer_required_invariants.is_empty()
        || output_currentness_facts.is_some()
    {
        return Err(
            WorthQueryProgramMigrationPreparationDenial::UnsupportedPosture(
                "program migration cannot participate in output production",
            ),
        );
    }

    let source_product = read_set.lease.product().observation().clone();
    let mutation_partition = application
        .issue_application_mutation_partition()
        .ok_or(WorthQueryProgramMigrationPreparationDenial::MutationPartitionUnavailable)?;
    let effect_count = effects.len();
    let batch = prepare_program_migration_batch(mutation_partition, &read_set.facts, effects)?;
    Ok((source_product, batch, effect_count))
}
