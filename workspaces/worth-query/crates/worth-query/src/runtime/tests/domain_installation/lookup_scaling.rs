use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UnrelatedDomain<const INDEX: usize>;

impl<const INDEX: usize> WorthQueryDomainEntryMarker for UnrelatedDomain<INDEX> {
    fn domain_key(&self) -> &'static str {
        unrelated_domain_name::<INDEX>()
    }

    fn display_name(&self) -> &'static str {
        unrelated_domain_name::<INDEX>()
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[]
    }
}

fn unrelated_domain_name<const INDEX: usize>() -> &'static str {
    match INDEX {
        0 => "WORTH.tests.unrelated-00",
        1 => "WORTH.tests.unrelated-01",
        2 => "WORTH.tests.unrelated-02",
        3 => "WORTH.tests.unrelated-03",
        4 => "WORTH.tests.unrelated-04",
        5 => "WORTH.tests.unrelated-05",
        6 => "WORTH.tests.unrelated-06",
        7 => "WORTH.tests.unrelated-07",
        _ => panic!("unsupported lookup-scaling fixture index"),
    }
}

fn unrelated_package<const INDEX: usize>() -> WorthQueryDomainPackage<UnrelatedDomain<INDEX>> {
    let name = unrelated_domain_name::<INDEX>()
        .strip_prefix("WORTH.tests.")
        .unwrap();
    let mut package = WorthQueryDomainPackage::declare(
        UnrelatedDomain::<INDEX>,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new(name).unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .requires_capability(WorthQueryCapabilityFamily::QueryRead);
    for operation_index in 0..8 {
        package = package.graph_read_operation(
            WorthQueryDomainGraphReadOperationDefinition::new(
                WorthQueryDomainIdentityName::new(format!("unrelated-{operation_index}")).unwrap(),
                1,
            )
            .accepts_relation(
                RelationName::new(format!("unrelated-{INDEX}-{operation_index}")).unwrap(),
            ),
        );
    }
    package
}

#[test]
fn installed_operation_lookup_width_is_independent_of_unrelated_packages_and_operations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let baseline = complete_backend_from_parts_builder()
        .domain_package(package(InstalledDomain))
        .unwrap()
        .build_backend_from_parts()
        .build(request_execution)
        .unwrap();
    assert_lookup_width(&baseline, 1);

    let builder = complete_backend_from_parts_builder()
        .domain_package(package(InstalledDomain))
        .unwrap()
        .domain_package(unrelated_package::<0>())
        .unwrap()
        .domain_package(unrelated_package::<1>())
        .unwrap()
        .domain_package(unrelated_package::<2>())
        .unwrap()
        .domain_package(unrelated_package::<3>())
        .unwrap()
        .domain_package(unrelated_package::<4>())
        .unwrap()
        .domain_package(unrelated_package::<5>())
        .unwrap()
        .domain_package(unrelated_package::<6>())
        .unwrap()
        .domain_package(unrelated_package::<7>())
        .unwrap();
    let runtime = builder
        .build_backend_from_parts()
        .build(request_execution)
        .unwrap();
    assert_lookup_width(&runtime, 65);
}

fn assert_lookup_width(runtime: &WorthQueryRuntime, expected_operation_count: usize) {
    let handle = runtime.domain(InstalledDomain).unwrap();
    let family = installed_operation_family(&handle);
    let before = runtime
        .domain_installation_lookup_counters()
        .indexed_operation_lookups();

    runtime
        .admit_graph_read_access_for_family(&family)
        .expect("the selected installed operation must resolve");

    let counters = runtime.domain_installation_lookup_counters();
    assert_eq!(counters.indexed_operation_lookups() - before, 1);
    assert_eq!(counters.package_content_scans(), 0);
    assert_eq!(
        runtime
            .verify_domain_execution_index_rebuild()
            .operation_count(),
        expected_operation_count
    );
}
