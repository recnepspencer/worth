#[path = "estate_operations/c4_retry_outbox_rebind.rs"]
pub mod c4_retry_outbox_rebind;
#[path = "estate_operations/disburse_estate.rs"]
pub mod disburse_estate;
#[path = "estate_operations/external_effect_dispatch.rs"]
pub mod external_effect_dispatch;
#[path = "estate_operations/freeze_account.rs"]
pub mod freeze_account;
#[path = "estate_operations/mutation_free_external_effect.rs"]
pub mod mutation_free_external_effect;
#[path = "estate_operations/notify_death.rs"]
pub mod notify_death;
#[path = "estate_operations/open_estate_case.rs"]
pub mod open_estate_case;
#[path = "estate_operations/phase8_cross_gate.rs"]
pub mod phase8_cross_gate;
#[path = "estate_operations/phase8_disposal.rs"]
pub mod phase8_disposal;
#[path = "estate_operations/phase8_exact_handle_authority.rs"]
pub mod phase8_exact_handle_authority;
#[path = "estate_operations/phase8_ordinary_commit_cost.rs"]
pub mod phase8_ordinary_commit_cost;
#[path = "estate_operations/phase8_publication_noninterference.rs"]
pub mod phase8_publication_noninterference;
#[path = "estate_operations/phase8_recovery_counters.rs"]
pub mod phase8_recovery_counters;
#[path = "estate_operations/phase8_recovery_expiry.rs"]
pub mod phase8_recovery_expiry;
#[path = "estate_operations/phase8_recovery_mechanism.rs"]
pub mod phase8_recovery_mechanism;
#[path = "estate_operations/phase8_recovery_policy.rs"]
pub mod phase8_recovery_policy;
#[path = "estate_operations/phase8_residue.rs"]
pub mod phase8_residue;
#[path = "estate_operations/phase8_safe_retry.rs"]
pub mod phase8_safe_retry;
#[path = "estate_operations/phase8_safe_retry_completed.rs"]
pub mod phase8_safe_retry_completed;
#[path = "estate_operations/phase8_safe_retry_pending.rs"]
pub mod phase8_safe_retry_pending;
#[path = "estate_operations/program_adoption_recovery.rs"]
pub mod program_adoption_recovery;
#[path = "estate_operations/recognize_executor.rs"]
pub mod recognize_executor;
#[path = "estate_operations/release_estate.rs"]
pub mod release_estate;
#[path = "estate_operations/undeclared_external_effect.rs"]
pub mod undeclared_external_effect;

#[path = "estate_operations/program_adoption_authorization.rs"]
pub mod program_adoption_authorization;

#[path = "estate_operations/owning_sources.rs"]
pub mod owning_sources;

// Estate sources retain the native original across progress, completion and refusal.
pub type EstateSourceAdvance<Intent, Action> =
    worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance<
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyOperation<
            bank_domain::schema::BankSchema,
            Intent,
            bank_server::BankApplication,
            Action,
            worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
        >,
        worth_query_host::facade::application_entry::WorthQueryCompletedSourceOperation<
            bank_domain::schema::BankSchema,
            Intent,
            bank_server::BankApplication,
            Action,
            worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
        >,
        worth_query_host::facade::application_entry::WorthQueryTerminalSourceOperation<
            bank_domain::schema::BankSchema,
            Intent,
            bank_server::BankApplication,
            Action,
            worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
        >,
    >;
