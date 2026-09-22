//! Exact target-binding ownership hostility for migration preparation.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramMigrationPreparationDenial, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use crate::bounded_dimension_model::dimension_entry::{
    PartDimensionWritten, PartDimensionWrittenBinding, SetPartDimensionDenial,
    SetPartDimensionDenialBinding, PART_IDENTITY,
};
use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::schema::SetPartDimensionInput;

type UnownedMigrationScope =
    worth_query_host::facade::declaration::application_operation::ApplicationMutationFieldScope<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
        crate::bounded_dimension_model::schema::Part,
        crate::bounded_dimension_model::schema::PartFacts,
        crate::bounded_dimension_model::schema::PartIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    >;

struct UnownedMigrationBinding;

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
    > for UnownedMigrationBinding
{
    type Input = SetPartDimensionInput;
    type InputBinding = crate::bounded_dimension_model::schema::SetPartDimensionInputBinding;
    type Result = PartDimensionWritten;
    type ResultBinding = PartDimensionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = crate::bounded_dimension_model::schema::SetPartDimension;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
        crate::bounded_dimension_model::schema::Part,
    >;
    type Denial = SetPartDimensionDenial;
    type DenialBinding = SetPartDimensionDenialBinding;
    type Output =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationOutputs;
    type ScopeBinding = UnownedMigrationScope;
    type PrincipalBinding = crate::bounded_dimension_model::schema::PartPrincipalBinding;
    type Mapping = crate::bounded_dimension_model::schema::ExternalMapping;
    type Principal = crate::bounded_dimension_model::schema::Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding =
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding;
    type SourceExpectation =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.unowned-migration-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.unowned-migration-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.unowned-migration-command.v1";
    const CANDIDATES: worth_query_host::facade::declaration::application_operation::ApplicationCandidateRequirements =
        worth_query_host::facade::declaration::application_operation::ApplicationCandidateRequirements::fixed_shape(
            worth_query_host::facade::declaration::application_operation::ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 0),
            worth_query_host::facade::declaration::application_operation::ApplicationCandidateResourceCeiling::bounded(0, 0),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        let mut identity = [0; 32];
        identity[..8].copy_from_slice(&key.to_le_bytes());
        identity
    }

    fn input_identity(input: &SetPartDimensionInput) -> [u8; 32] {
        let mut identity = [0; 32];
        identity[..8].copy_from_slice(&input.dimension.to_le_bytes());
        identity
    }

    fn scope_field(
    ) -> worth_query_host::facade::declaration::application_schema::ApplicationFieldRef<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
        crate::bounded_dimension_model::schema::Part,
        crate::bounded_dimension_model::schema::PartFacts,
        crate::bounded_dimension_model::schema::PartIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::EqualityPredicate,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    > {
        crate::bounded_dimension_model::schema::PartIdentityField::reference()
    }

    fn principal_binding() -> worth_query_host::facade::declaration::application_schema::ApplicationPrincipalBindingRef<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
        crate::bounded_dimension_model::schema::PartPrincipalBinding,
        crate::bounded_dimension_model::schema::ExternalMapping,
        crate::bounded_dimension_model::schema::Principal,
        u64,
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding,
    >{
        crate::bounded_dimension_model::schema::PartPrincipalBinding::reference()
    }
}

#[derive(Clone)]
struct UnownedMigrationIntent {
    input: SetPartDimensionInput,
}

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<
        crate::bounded_dimension_model::schema::BoundedDimensionSchema,
    > for UnownedMigrationIntent
{
    type Binding = UnownedMigrationBinding;

    fn input(&self) -> &SetPartDimensionInput {
        &self.input
    }

    fn scope_binding(&self) -> UnownedMigrationScope {
        UnownedMigrationScope::new(
            crate::bounded_dimension_model::schema::PartIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}

#[test]
fn target_program_must_own_the_exact_migration_binding() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(UnownedMigrationIntent {
            input: SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 15,
            },
        })
        .without_source()
        .idempotency(&0x9175_2100)
        .prepare_program_migration(&target)
        .err()
        .expect("operation identity cannot substitute for exact binding ownership");
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramMigrationPreparationDenial::Migration(
            worth_query_host::facade::primary_graph::WorthQueryProgramMigrationPreparationDenial::TargetBindingUnowned
        )
    ));
}
