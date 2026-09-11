use bank_domain::{
    estate::{EmergencyAccessStatus, RestrictedBankField},
    queries::EstateEmergencyAccessActivityQuery,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        CompleteEstateMandatoryReviewCapability, CompleteEstateMandatoryReviewOperation,
        EmergencyAccessStatusBinding, EstateEmergencyAccessActivityEventBinding,
        RequestEstateEmergencyAccessCapability, RequestEstateEmergencyAccessOperation,
        RestrictedBankFieldBinding, RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation, ViewEstateAdministrationCapability,
        ViewEstateEmergencyProtectionCapability, ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    declaration::{
        application_capability::ApplicationCapabilityValidityTimeline,
        application_query::{
            ApplicationQueryDisclosurePosture, ApplicationQueryObservableInfluence,
            ApplicationQueryOrderingDirection, ApplicationQueryResultTraversalDirection,
        },
        application_schema::{ApplicationScalarValueBinding, ApplicationStructuredValueBinding},
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

use super::installed_bank;
use worth_query_host::facade::domain::WorthQueryOperationTouchScope;

#[test]
fn emergency_view_installs_exact_resource_lifecycle_and_effect_meaning() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            ViewEstateEmergencyProtectionCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let elevation = capability
        .contract()
        .elevation()
        .definition()
        .expect("emergency protection view is elevation governed");

    assert_eq!(
        elevation.states().expired().value(),
        &EmergencyAccessStatusBinding::encode(&EmergencyAccessStatus::Expired).unwrap()
    );
    assert_eq!(
        elevation.validity().timeline(),
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds
    );
    assert_eq!(
        (
            elevation.validity().not_before().field(),
            elevation.validity().not_after().field(),
        ),
        (
            "EmergencyAccessIssuedAtField",
            "EmergencyAccessExpiresAtField",
        )
    );
    assert_eq!(
        elevation
            .resource_relation()
            .expect("elevation retains its direct estate relation")
            .relation(),
        "EmergencyEstate"
    );
    let lifecycle = elevation.lifecycle();
    assert_eq!(
        (
            lifecycle.elevation_slot().slot(),
            lifecycle.review_slot().slot()
        ),
        ("EstateEmergencyAccessSlot", "EstateMandatoryReviewSlot")
    );
    assert_eq!(
        lifecycle
            .transitions()
            .map(|transition| transition.operation().operation()),
        [
            "RequestEstateEmergencyAccessOperation",
            "ApproveEstateEmergencyAccessOperation",
            "RevokeEstateEmergencyAccessOperation",
            "CompleteEstateMandatoryReviewOperation",
        ]
    );
    assert_eq!(
        lifecycle
            .transitions()
            .map(|transition| transition.capability()),
        [
            "RequestEstateEmergencyAccessCapability",
            "ApproveEstateEmergencyAccessCapability",
            "RevokeEstateEmergencyAccessCapability",
            "CompleteEstateMandatoryReviewCapability",
        ]
    );
    assert!(lifecycle.transitions().iter().all(|transition| {
        transition
            .lifecycle_effect()
            .is_some_and(|effect| effect.effect() == "EstateEmergencyAccessActivityEffect")
    }));
}

#[test]
fn emergency_access_activity_installs_one_identity_across_all_five_lanes() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let query = bank
        .application_query(EstateEmergencyAccessActivityQuery::reference())
        .unwrap();

    let basis = query.basis_support();
    assert!(basis.current());
    assert!(basis.pinned());
    assert!(basis.preview());
    let lanes = query.lanes();
    assert!(lanes.one_shot_enabled());
    assert!(lanes.historical_enabled());
    assert!(lanes.preview_enabled());
    assert!(lanes.live_enabled());

    let continuation = query
        .continuation()
        .expect("the naturally many emergency-access relation is resumable");
    assert_eq!(continuation.relation(), "EmergencyEstate");
    assert_eq!(continuation.parent_entity(), "EstateCase");
    assert_eq!(continuation.child_entity(), "EmergencyAccess");
    assert_eq!(
        continuation.direction(),
        ApplicationQueryResultTraversalDirection::Reverse
    );
    assert_eq!(
        continuation
            .ordering()
            .iter()
            .map(|ordering| (ordering.field().2, ordering.direction()))
            .collect::<Vec<_>>(),
        vec![
            (
                "EmergencyAccessIssuedAtField",
                ApplicationQueryOrderingDirection::Ascending,
            ),
            (
                "EmergencyAccessIdentityField",
                ApplicationQueryOrderingDirection::Ascending,
            ),
        ]
    );

    let live = query
        .live()
        .expect("activity declares its exact live cause");
    assert_eq!(live.effect(), "EstateEmergencyAccessActivityEffect");
    assert_eq!(
        live.payload_type(),
        EstateEmergencyAccessActivityEventBinding::IDENTITY_NAME
    );
    assert_eq!(live.collection_path(), continuation.collection_path());
    assert_eq!(live.scope_identity().field(), "EstateCaseIdentityField");
    assert_eq!(
        live.target_identity().field(),
        "EmergencyAccessIdentityField"
    );
}

#[test]
fn emergency_access_activity_installs_only_governed_lifecycle_disclosure() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let query = bank
        .application_query(EstateEmergencyAccessActivityQuery::reference())
        .unwrap();
    let disclosure = query.disclosure();

    assert_eq!(
        disclosure.posture(),
        ApplicationQueryDisclosurePosture::Governed
    );
    assert_eq!(
        disclosure.capability_name(),
        Some(ViewEstateEmergencyProtectionCapability::reference().name())
    );
    assert_eq!(disclosure.rules().len(), 13);
    let expected_disclosure =
        RestrictedBankFieldBinding::encode(&RestrictedBankField::EmergencyAccessActivity).unwrap();
    assert!(disclosure
        .rules()
        .iter()
        .all(|rule| rule.disclosure_value() == &expected_disclosure));
    assert_field_influences(
        disclosure,
        "EstateCaseIdentityField",
        &[ApplicationQueryObservableInfluence::LiveMembership],
        2,
    );
    let ordering_influences = [
        ApplicationQueryObservableInfluence::Ordering,
        ApplicationQueryObservableInfluence::Pagination,
        ApplicationQueryObservableInfluence::HistoricalMembership,
        ApplicationQueryObservableInfluence::Preview,
        ApplicationQueryObservableInfluence::LiveMembership,
    ];
    assert_field_influences(
        disclosure,
        "EmergencyAccessIdentityField",
        &ordering_influences,
        2,
    );
    assert_field_influences(
        disclosure,
        "EmergencyAccessIssuedAtField",
        &ordering_influences,
        2,
    );
    assert_eq!(
        relation_influences(disclosure, "EmergencyEstate"),
        vec![ApplicationQueryObservableInfluence::Pagination]
    );
    for field in [
        "EmergencyAccessReasonField",
        "EmergencyAccessStatusField",
        "EmergencyAccessExpiresAtField",
        "MandatoryReviewIdentityField",
        "MandatoryReviewStatusField",
    ] {
        assert_field_influences(disclosure, field, &[], 1);
    }
    assert!(relation_influences(disclosure, "EmergencyReview").is_empty());

    let mut projected_fields = query
        .read_graph()
        .projections()
        .iter()
        .map(|projection| projection.field())
        .collect::<Vec<_>>();
    projected_fields.sort_unstable();
    assert_eq!(
        projected_fields,
        vec![
            "EmergencyAccessExpiresAtField",
            "EmergencyAccessIdentityField",
            "EmergencyAccessIssuedAtField",
            "EmergencyAccessReasonField",
            "EmergencyAccessStatusField",
            "EstateCaseIdentityField",
            "MandatoryReviewIdentityField",
            "MandatoryReviewStatusField",
        ]
    );
    assert!(!projected_fields.contains(&"PrincipalIdentityField"));
}

#[test]
fn activity_field_is_permitted_only_by_emergency_protection() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let emergency = bank
        .capability(
            ViewEstateEmergencyProtectionCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let administration = bank
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let field =
        RestrictedBankFieldBinding::encode(&RestrictedBankField::EmergencyAccessActivity).unwrap();

    assert!(disclosure_values(&emergency).contains(&field));
    assert!(!disclosure_values(&administration).contains(&field));
}

#[test]
fn every_lifecycle_program_installs_one_exact_activity_emission() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());

    macro_rules! assert_emission {
        ($capability:ty, $operation:ty) => {{
            let _capability = bank
                .capability(<$capability>::reference(), <$operation>::reference())
                .unwrap();
            let operation = bank.installed_operation(<$operation>::reference()).unwrap();
            assert_eq!(
                operation
                    .contracts()
                    .emissions()
                    .emissions()
                    .iter()
                    .map(|emission| emission.effect())
                    .collect::<Vec<_>>(),
                vec!["EstateEmergencyAccessActivityEffect"]
            );
            operation.contracts().touches().clone()
        }};
    }

    let request = assert_emission!(
        RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation
    );
    assert!(request.scopes().iter().any(|target| matches!(
        target,
        WorthQueryOperationTouchScope::LinkRelation(scope)
            if scope.relation() == "EmergencyEstate" && scope.from() == "EmergencyAccess" && scope.to() == "EstateCase"
    )));
    let _ = assert_emission!(
        ApproveEstateEmergencyAccessCapability,
        ApproveEstateEmergencyAccessOperation
    );
    let _ = assert_emission!(
        RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation
    );
    let _ = assert_emission!(
        CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation
    );
}

fn assert_field_influences(
    disclosure: &worth_query_host::facade::declaration::application_query::ApplicationQueryDisclosureContract,
    field: &str,
    expected: &[ApplicationQueryObservableInfluence],
    expected_rule_count: usize,
) {
    let matching = disclosure
        .rules()
        .iter()
        .filter(|rule| {
            rule.selector()
                .field_contract()
                .is_some_and(|(_, _, candidate)| candidate == field)
        })
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), expected_rule_count, "field {field}");
    for rule in matching {
        assert_eq!(
            rule.influence().permitted().collect::<Vec<_>>(),
            expected,
            "field {field}"
        );
    }
}

fn relation_influences(
    disclosure: &worth_query_host::facade::declaration::application_query::ApplicationQueryDisclosureContract,
    relation: &str,
) -> Vec<ApplicationQueryObservableInfluence> {
    disclosure
        .rules()
        .iter()
        .find(|rule| {
            rule.selector()
                .relation_contract()
                .is_some_and(|(candidate, ..)| candidate == relation)
        })
        .expect("relation has one governed disclosure rule")
        .influence()
        .permitted()
        .collect()
}

fn disclosure_values<Schema, Capability, Operation, Input>(
    capability: &worth_query_host::facade::domain::WorthQueryInstalledApplicationCapability<
        Schema,
        Capability,
        Operation,
        Input,
    >,
) -> Vec<worth_foundational::facade::AspectValue>
where
    Schema: worth_query_host::facade::declaration::application_schema::ApplicationSchema,
{
    let worth_query_host::facade::declaration::application_capability::ApplicationCapabilityDisclosureRule::Permit(guards) =
        capability.contract().composition().propagation().disclosure()
    else {
        panic!("view capability must install an explicit disclosure matrix");
    };
    guards[0].requirements()[0].values().to_vec()
}
