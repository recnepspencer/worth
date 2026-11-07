use super::*;

#[test]
fn installed_read_projection_and_receipts_carry_one_authority_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime.workspace("installed-domain-read").unwrap();

    let completion = handle
        .read(identity_read)
        .unwrap()
        .using(current())
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result()
        .unwrap();
    assert_eq!(
        completion.receipt().capability(),
        WorthQueryInstalledDomainCapabilityKind::Read
    );
    assert_eq!(
        completion
            .receipt()
            .installed_authority()
            .witness_identity(),
        &expected
    );

    let projection = completion.project(project_facts().entity_identities());
    assert_eq!(
        projection.receipt().capability(),
        WorthQueryInstalledDomainCapabilityKind::Projection
    );
    assert_eq!(
        projection
            .receipt()
            .installed_authority()
            .witness_identity(),
        &expected
    );
}
