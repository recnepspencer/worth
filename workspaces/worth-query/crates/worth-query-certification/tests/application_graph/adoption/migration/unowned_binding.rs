//! Exact target-binding ownership hostility for migration preparation.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramMigrationPreparationDenial, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::programs::RetentionProgramP1;
use crate::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
use crate::document_retention_model::schema::SetRetentionInput;

type UnownedMigrationScope =
    worth_query_host::facade::declaration::application_operation::ApplicationMutationFieldScope<
        crate::document_retention_model::schema::DocumentRetentionSchema,
        crate::document_retention_model::schema::Document,
        crate::document_retention_model::schema::DocumentFacts,
        crate::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    >;

struct UnownedMigrationBinding;

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        crate::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationBinding
{
    type Input = SetRetentionInput;
    type InputBinding = crate::document_retention_model::schema::SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = crate::document_retention_model::schema::SetRetention;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        crate::document_retention_model::schema::DocumentRetentionSchema,
        crate::document_retention_model::schema::Document,
    >;
    type Denial = SetRetentionDenial;
    type DenialBinding = SetRetentionDenialBinding;
    type Output =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationOutputs;
    type ScopeBinding = UnownedMigrationScope;
    type PrincipalBinding = crate::document_retention_model::schema::DocumentPrincipalBinding;
    type Mapping = crate::document_retention_model::schema::ExternalMapping;
    type Principal = crate::document_retention_model::schema::Principal;
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
            worth_query_host::facade::declaration::application_operation::ApplicationCandidateResourceCeiling::representation_bytes(0),
        );

    fn scope_field(
    ) -> worth_query_host::facade::declaration::application_schema::ApplicationFieldRef<
        crate::document_retention_model::schema::DocumentRetentionSchema,
        crate::document_retention_model::schema::Document,
        crate::document_retention_model::schema::DocumentFacts,
        crate::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::EqualityPredicate,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    > {
        crate::document_retention_model::schema::DocumentIdentityField::reference()
    }

    fn principal_binding() -> worth_query_host::facade::declaration::application_schema::ApplicationPrincipalBindingRef<
        crate::document_retention_model::schema::DocumentRetentionSchema,
        crate::document_retention_model::schema::DocumentPrincipalBinding,
        crate::document_retention_model::schema::ExternalMapping,
        crate::document_retention_model::schema::Principal,
        u64,
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding,
    >{
        crate::document_retention_model::schema::DocumentPrincipalBinding::reference()
    }
}

#[derive(Clone)]
struct UnownedMigrationIntent {
    input: SetRetentionInput,
}

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<
        crate::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationIntent
{
    type Binding = UnownedMigrationBinding;

    fn input(&self) -> &SetRetentionInput {
        &self.input
    }

    fn scope_binding(&self) -> UnownedMigrationScope {
        UnownedMigrationScope::new(
            crate::document_retention_model::schema::DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}

#[test]
fn target_program_must_own_the_exact_migration_binding() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .mutate(UnownedMigrationIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 15,
                    },
                })
                .without_source()
                .idempotency(&0x9175_2100)
                .prepare_program_migration(
                    &phase,
                    &target,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("operation identity cannot substitute for exact binding ownership")
        })
        .expect("the installed host admits migration preparation");
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramMigrationPreparationDenial::Migration(
            worth_query_host::facade::primary_graph::WorthQueryProgramMigrationPreparationDenial::TargetBindingUnowned
        )
    ));
}
