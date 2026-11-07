//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphApplicationRuntime<()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime.rs", "WorthQueryPrimaryGraphApplicationRuntime", 128 * 1024);
        owner.field::<conditional_cleanup::ConditionalRuntimeCleanup<()>>("conditional_cleanup");
        owner.field::<WorthQueryInstalledApplicationSchema<()>>("installed_schema");
        owner.field::<WorthQueryApplicationRuntimeCore>("runtime");
        owner.field::<teardown::ApplicationPostConditionalTeardown>("post_conditional_teardown");
        owner.field::<super::super::handler::InstalledMutationHandlerRegistry<()>>(
            "mutation_handlers",
        );
        owner.field::<super::super::WorthQueryApplicationInvariantProjectionAuthority<()>>(
            "mutation_projection",
        );
        owner.field::<super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<()>>("installed_producers");
        owner.field::<teardown::ApplicationOutputTeardown>("output_teardown");
        owner.field::<std::collections::HashSet<TypeId>>("program_required_bindings");
        owner.field::<std::collections::HashSet<TypeId>>("program_required_operations");
        owner.field::<std::collections::HashSet<TypeId>>("workflow_guarded_operations");
        owner.field::<Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<()>>>("program_support");
        owner.field::<std::sync::Mutex<std::collections::BTreeMap<worth_query_declaration::facade::application_program::ApplicationProgramRevision, worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner<()>>>>("signing_owners");
        owner.field::<super::super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<()>>("installed_conditionals");
        owner.field::<super::super::application_query::managed_index::ManagedIndexProviderCatalog<()>>("managed_indexes");
        owner.field::<teardown::ApplicationWorkflowTeardown>("workflow_teardown");
    }
}
