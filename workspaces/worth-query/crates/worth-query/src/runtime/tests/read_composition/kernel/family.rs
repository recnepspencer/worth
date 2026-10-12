use super::super::support::*;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField, TraversalSelector};
use crate::runtime::{
    WorthQueryReadFamily, WorthQueryReadFamilyAdmission, WorthQueryReadGraphFamily,
    WorthQueryReadScopeClass,
};

#[test]
fn define_read_family_freezes_reusable_read_graph_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.family-define")
        .expect("read-backed runtime should open a workspace");

    let family = workspace
        .define_read_family("manager-chain", |read| {
            read.anchored_detail(
                "user",
                expanded_manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("manager", 2)
                                .expect("bounded traversal should build"),
                        )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect("read family should define");

    assert_eq!(family.family_name(), "manager-chain");
    assert!(!family.family_digest().is_empty());
    assert_eq!(
        family.read_graph().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
    assert_eq!(
        family.read_graph().family(),
        &WorthQueryReadGraphFamily::Detail
    );
    assert_eq!(
        family.admission(),
        &WorthQueryReadFamilyAdmission::KernelOnly
    );
}

#[test]
fn execute_read_family_reuses_same_canonical_read_graph_across_runs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.family-execute")
        .expect("read-backed runtime should open a workspace");

    let family = workspace
        .define_read_family("manager-chain", |read| {
            read.anchored_detail(
                "user",
                expanded_manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("manager", 2)
                                .expect("bounded traversal should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new(
                                "profile",
                                "display_name",
                                "display_name",
                            )
                            .expect("name result-shape field should build"),
                        )
                },
            )
        })
        .expect("read family should define");

    let first = workspace
        .execute_read_family(request_execution, &family)
        .expect("first read-family execution should succeed");
    let second = workspace
        .execute_read_family(request_execution, &family)
        .expect("second read-family execution should succeed");

    assert_eq!(
        first.receipt().read_graph_digest(),
        family.read_graph().digest()
    );
    assert_eq!(
        second.receipt().read_graph_digest(),
        family.read_graph().digest()
    );
    assert_eq!(
        first.receipt().query_digest(),
        second.receipt().query_digest()
    );
    assert_eq!(
        first.receipt().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
}

fn _read_family_type_is_public(_family: &WorthQueryReadFamily) {}
