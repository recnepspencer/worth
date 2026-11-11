//! Coupled correction meaning reaches the actual Directory material guard.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;

struct DirectoryProgram<const MODE: u8>;
impl<const MODE: u8> ApplicationProgramDefinition<Model<MODE>> for DirectoryProgram<MODE>
where
    RelationalContribution: WorthQueryApplicationContribution<Model<MODE>, Configuration = ()>,
{
    type Contributions = (RelationalContribution,);
    type Outputs = <program::Program as ApplicationProgramDefinition<Model<MODE>>>::Outputs;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.directory-coupled-refusal.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <program::Program as ApplicationProgramDefinition<Model<MODE>>>::feature_specs()
    }
}
fn refuses<const MODE: u8>(
    expected: worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial,
) where
    RelationalContribution: WorthQueryApplicationContribution<Model<MODE>, Configuration = ()>,
{
    use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind as Kind;
    let path = std::env::temp_dir().join(format!(
        "worth-coupled-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let budget = worth_execution::SerialMemoryBudget::new(Some(256 << 20));
    let cancel = worth_execution::CancellationSource::new();
    let request = worth_execution::SerialRequest::from_memory(budget, cancel.token(), None);
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let open = || {
        let schema = Model::<MODE>::declaration().unwrap();
        let request = WorthQueryApplicationHomeRequest::new(WorthQueryApplicationHomeLocation::Directory(path.clone()), ApplicationProgramAuthoring::<Model<MODE>, DirectoryProgram<MODE>>::begin().with_schema(&schema).validated_program().unwrap(), WorthQueryApplicationProgramRoster::new(), schema, ((),), ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(), host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), |_, graph, schema| graph.bind_principal(&schema.principal_binding(actor()).unwrap(), WorthQueryApplicationPrincipalKey::new("operator").unwrap(), 1, operator_identity(), WorthQueryPrincipalMappingStatus::Enabled));
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(request.open_in_advancement(&phase))
        })
        .unwrap()
        .unwrap_or_else(|cause| panic!("actual coupled Home: {cause:?}"))
    };
    let home = open();
    let before = home.runtime().runtime().current_world();
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let denied = home
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(lifecycle::Create {
                create_co: false,
                scope: 1,
                id: 40,
                depth: 1,
                co_anchors: Vec::new(),
            })
            .without_source()
            .idempotency(&"coupled-original".to_owned())
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .err()
            .expect("coupled outputs have no durable continuation yet");
        assert!(matches!(denied.kind(), Kind::DurableStateAdmission(cause) if cause == expected));
        assert_eq!(home.runtime().runtime().current_world(), before);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no original publication: {cause:?}"))
        .reclaim();
    let home = open();
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("coupled refusal reopen: {cause:?}"))
        .reclaim();
}

#[test]
fn directory_coupled_correction_refuses_before_original_handler_or_world_publication() {
    refuses::<60>(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionCoupledOutputs);
}
#[test]
fn directory_authored_output_correction_refuses_before_original_handler_or_world_publication() {
    refuses::<0>(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionOutputMaterial);
}
