use std::sync::Mutex;

use crate::contribution_composed_orchestration::WorthQueryContributionIntent;
use crate::domain_capabilities::{
    WorthQueryExplanationContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::grouped_authoring::worth_query_grouped_declaration_checked_on_handle;
use crate::grouped_authoring::{
    WorthQueryGroupedContributionStop, WorthQueryGroupedDeclarationChecked,
};

use super::support::{
    admitted_handle, admitted_handle_with_shifted_relational_digest,
    counting_geometry_canonicalization_count, reset_counting_geometry_canonicalization_count,
    CountingGeometryInput, GeometryInput,
};

static COUNTING_GEOMETRY_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn grouped_contributions_reuse_member_progression_after_group_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _guard = COUNTING_GEOMETRY_TEST_LOCK
        .lock()
        .expect("counting grouped test lock should remain available");
    let handle = admitted_handle(request_execution, "main");
    reset_counting_geometry_canonicalization_count();

    let result = handle.grouped_contributions_checked(
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(CountingGeometryInput::new("face-a"))
            .with_member(CountingGeometryInput::new("face-b"))
            .with_shared_support_contribution(
                WorthQuerySupportContributionAuthoring::declaration_support(
                    "support.shared",
                    "shared grouped support",
                ),
            ),
    );

    assert!(result.is_ok(), "grouped contributions should admit");
    assert_eq!(
        counting_geometry_canonicalization_count(),
        2,
        "grouped contributions should canonicalize each member once during grouped admission, not rebuild member declaration work during contribution lowering",
    );
}

#[test]
fn grouped_contributions_from_admitted_declaration_reuse_grouped_authority_without_recanonicalizing(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _guard = COUNTING_GEOMETRY_TEST_LOCK
        .lock()
        .expect("counting grouped test lock should remain available");
    let handle = admitted_handle(request_execution, "main");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(CountingGeometryInput::new("face-a"))
            .with_member(CountingGeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit before contribution composition")
        }
    };

    reset_counting_geometry_canonicalization_count();
    let result =
        handle.grouped_contributions_checked(declaration.with_shared_support_contribution(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "support.shared",
                "shared grouped support",
            ),
        ));

    assert!(
        result.is_ok(),
        "grouped contributions should admit from declaration"
    );
    assert_eq!(
        counting_geometry_canonicalization_count(),
        0,
        "grouped contributions built from an admitted grouped declaration should reuse grouped authority instead of rebuilding declaration canonicalization",
    );
}

#[test]
fn grouped_contributions_from_admitted_declaration_match_direct_grouped_path_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let member_explanation = WorthQueryContributionIntent::explanation(
        WorthQueryExplanationContributionAuthoring::explains_fallback(
            "explain.member",
            "member-local fallback explanation",
        ),
    );
    let direct = match handle.grouped_contributions_checked(
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b"))
            .with_shared_support_contribution(
                WorthQuerySupportContributionAuthoring::declaration_support(
                    "support.shared",
                    "shared grouped support",
                ),
            )
            .with_member_contribution(1, member_explanation.clone()),
    ) {
        Ok(value) => value,
        Err(_) => panic!("direct grouped contribution composition should admit"),
    };
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit before reusable composition")
        }
    };
    let reusable = match handle.grouped_contributions_checked(
        declaration
            .with_shared_support_contribution(
                WorthQuerySupportContributionAuthoring::declaration_support(
                    "support.shared",
                    "shared grouped support",
                ),
            )
            .with_member_contribution(1, member_explanation),
    ) {
        Ok(value) => value,
        Err(_) => panic!("reusable grouped contribution composition should admit"),
    };

    assert_eq!(
        direct.declaration().group_digest(),
        reusable.declaration().group_digest(),
        "direct and reusable grouped contribution paths must preserve canonical grouped identity",
    );
    assert_eq!(direct.members().len(), reusable.members().len());
    assert_eq!(
        direct.members()[0].0.member_contribution_count(),
        reusable.members()[0].0.member_contribution_count()
    );
    assert_eq!(
        direct.members()[1].0.member_contribution_count(),
        reusable.members()[1].0.member_contribution_count()
    );
    assert_eq!(
        direct.members()[0].1.composition_for_reporting(),
        reusable.members()[0].1.composition_for_reporting()
    );
    assert_eq!(
        direct.members()[1].1.composition_for_reporting(),
        reusable.members()[1].1.composition_for_reporting()
    );
}

#[test]
fn grouped_contributions_from_admitted_declaration_reject_wrong_world_before_member_lowering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _guard = COUNTING_GEOMETRY_TEST_LOCK
        .lock()
        .expect("counting grouped test lock should remain available");
    let left = admitted_handle(request_execution, "main");
    let right = admitted_handle(request_execution, "other");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &left,
        left.geometry_helpers()
            .local_neighborhood_for_active_face_selection(CountingGeometryInput::new("face-a"))
            .with_member(CountingGeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit before wrong-world composition")
        }
    };

    reset_counting_geometry_canonicalization_count();
    match right.grouped_contributions_checked(declaration.with_shared_support_contribution(
        WorthQuerySupportContributionAuthoring::declaration_support(
            "support.shared",
            "shared grouped support",
        ),
    )) {
        Err(WorthQueryGroupedContributionStop::WrongWorld(stop)) => {
            assert_eq!(
                stop.reason(),
                "the grouped declaration was admitted in a different operating context"
            );
            assert_eq!(counting_geometry_canonicalization_count(), 0);
        }
        _ => panic!("expected grouped wrong-world contribution stop"),
    }
}

#[test]
fn grouped_contributions_from_admitted_declaration_reject_wrong_handle_before_member_lowering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _guard = COUNTING_GEOMETRY_TEST_LOCK
        .lock()
        .expect("counting grouped test lock should remain available");
    let left = admitted_handle(request_execution, "main");
    let right = admitted_handle_with_shifted_relational_digest(request_execution, "main");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &left,
        left.geometry_helpers()
            .local_neighborhood_for_active_face_selection(CountingGeometryInput::new("face-a"))
            .with_member(CountingGeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit before wrong-handle composition")
        }
    };

    reset_counting_geometry_canonicalization_count();
    match right.grouped_contributions_checked(declaration.with_shared_support_contribution(
        WorthQuerySupportContributionAuthoring::declaration_support(
            "support.shared",
            "shared grouped support",
        ),
    )) {
        Err(WorthQueryGroupedContributionStop::WrongHandle(stop)) => {
            assert_eq!(
                stop.reason(),
                "the grouped declaration was admitted on a different configured domain handle"
            );
            assert_eq!(counting_geometry_canonicalization_count(), 0);
        }
        _ => panic!("expected grouped wrong-handle contribution stop"),
    }
}
