use super::*;
use crate::ordinary::read::current;
use crate::runtime::WorthQueryLiveGraphReadMaintenanceBudget;

#[test]
fn installed_operation_resolution_is_identical_across_public_runtime_paths() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let operation = installed_operation_declaration(&handle);
    let family = installed_bound_operation_family(operation.clone());
    let lookup_baseline = runtime
        .domain_installation_lookup_counters()
        .indexed_operation_lookups();

    let admission = runtime
        .admit_graph_read_access_for_family(&family)
        .expect("ordinary admission must resolve the installed declaration");
    let mut workspace = runtime
        .workspace("installed-operation-equivalence")
        .unwrap();
    let ordinary_explanation = workspace
        .explain_graph_read_access_shape(&family)
        .expect("ordinary explanation must resolve the installed declaration");
    let live_plan = workspace
        .plan_live_graph_read_access(&family, WorthQueryLiveGraphReadMaintenanceBudget::bounded())
        .expect("live maintenance must resolve the installed declaration");
    let ordinary_completion = handle
        .read(|read| installed_operation_read(read, operation))
        .expect("the handle-bound ordinary read must declare")
        .using(current())
        .run(request_execution, &mut workspace)
        .expect("the handle and workspace share runtime authority")
        .into_result()
        .expect("the installed operation must execute through ordinary read");

    let expected_shape = admission
        .requirement_set()
        .access_shape_digest()
        .render_hex();
    assert_eq!(
        ordinary_explanation.access_shape().digest().as_str(),
        expected_shape.as_str()
    );
    assert_eq!(live_plan.one_shot_access_shape_digest(), expected_shape);
    assert_eq!(
        ordinary_completion
            .receipt()
            .installed_authority()
            .witness_identity(),
        handle.authority_witness().witness_identity()
    );

    let counters = workspace.runtime.domain_installation_lookup_counters();
    assert_eq!(counters.indexed_operation_lookups() - lookup_baseline, 4);
    assert_eq!(counters.package_content_scans(), 0);
}

fn installed_operation_declaration(
    handle: &crate::domain_installation::WorthQueryInstalledDomainHandle<InstalledDomain>,
) -> crate::domain_installation::WorthQueryInstalledGraphReadOperation {
    handle.graph_read_operation(
        &WorthQueryDomainGraphReadOperationDefinition::new(
            WorthQueryDomainIdentityName::new("neighbors").unwrap(),
            1,
        )
        .accepts_relation(RelationName::new("manager").unwrap()),
    )
}

fn installed_operation_read(
    read: WorthQueryReadBuilder<crate::ordinary::read::WorthQueryDeclaredReadIntent>,
    operation: crate::domain_installation::WorthQueryInstalledGraphReadOperation,
) -> Result<crate::ordinary::read::WorthQueryDeclaredReadIntent, crate::runtime::WorthQueryReadDenial>
{
    read.local_detail_with_installed_operation(
        operation,
        "user",
        schema(),
        |query: DetailQueryBuilder| {
            query.project(AspectFieldSelector::new("identity", "id").unwrap())
        },
        |shape: DetailResultShapeBuilder| {
            shape.field(
                crate::authoring::AuthoredResultShapeField::new("identity", "id", "id").unwrap(),
            )
        },
    )
}
