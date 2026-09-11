use bank_domain::{
    estate::{CapabilityGrantStatus, RestrictedBankField},
    model::EmployeeRole,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        CapabilityGrantStatusBinding, CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation, EmployeeRoleBinding,
        RecognizeEstateExecutorCapability, RecognizeEstateExecutorOperation,
        ReleaseEstateCapability, ReleaseEstateOperation, RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation, RestrictedBankFieldBinding,
        ViewEstateAdministrationCapability, ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    declaration::{
        application_capability::{
            ApplicationCapabilityDisclosureRule, ApplicationCapabilityGraphRule,
        },
        application_schema::ApplicationScalarValueBinding,
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

use super::installed_bank;

#[test]
fn release_contract_installs_exact_roles_conflict_and_executor_separation() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            ReleaseEstateCapability::reference(),
            ReleaseEstateOperation::reference(),
        )
        .unwrap();
    let composition = capability.contract().composition();

    let allow = composition.decision().allow().graph();
    assert_eq!(requirement_widths(allow), vec![3]);
    let mut roles = all_clauses(allow)
        .iter()
        .flat_map(|clause| clause.path().predicates())
        .filter(|predicate| predicate.field() == "AssignmentRole")
        .map(|predicate| predicate.value().clone())
        .collect::<Vec<_>>();
    roles.sort();
    assert_eq!(
        roles,
        expected_roles([
            EmployeeRole::EstateSpecialist,
            EmployeeRole::Compliance,
            EmployeeRole::Legal,
        ])
    );
    assert_eq!(
        path_relations(composition.decision().conflict().graph().unwrap()),
        vec!["EstateBeneficiary"]
    );
    assert_eq!(
        path_relations(composition.actors().separation_of_duty().graph().unwrap()),
        vec!["EstateExecutor"]
    );
}

#[test]
fn estate_view_contract_installs_exact_role_and_disclosure_composition() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let composition = capability.contract().composition();

    let allow = composition.decision().allow().graph();
    assert_eq!(requirement_widths(allow), vec![4]);
    assert_eq!(
        all_clauses(allow)
            .iter()
            .map(|clause| clause.path().predicates().len())
            .sum::<usize>(),
        4
    );
    assert!(composition.decision().deny().graph().is_none());
    let conflict = composition
        .decision()
        .conflict()
        .graph()
        .expect("estate disclosure declares beneficiary conflict");
    assert_eq!(path_relations(conflict), vec!["EstateBeneficiary"]);
    assert_eq!(
        composition
            .propagation()
            .delegation()
            .maximum_depth()
            .expect("estate delegation must retain its installed depth")
            .maximum(),
        8
    );

    let ApplicationCapabilityDisclosureRule::Permit(guards) =
        composition.propagation().disclosure()
    else {
        panic!("estate view field scope must carry an explicit disclosure matrix");
    };
    assert_eq!(guards.len(), 1);
    assert_eq!(guards[0].requirements().len(), 1);
    assert_eq!(
        guards[0].requirements()[0].values(),
        &expected_values([
            RestrictedBankField::CustomerIdentity,
            RestrictedBankField::BeneficiaryIdentity,
            RestrictedBankField::AccountDetails,
            RestrictedBankField::PostingHistory,
            RestrictedBankField::GovernanceMetadata,
        ])
    );
}

#[test]
fn emergency_approval_contract_installs_conflict_and_requester_separation() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            ApproveEstateEmergencyAccessCapability::reference(),
            ApproveEstateEmergencyAccessOperation::reference(),
        )
        .unwrap();
    let composition = capability.contract().composition();

    let allow = composition.decision().allow().graph();
    assert_eq!(requirement_widths(allow), vec![3]);
    assert!(all_clauses(allow)
        .iter()
        .all(|clause| clause.context_anchors().is_empty()));
    assert_eq!(
        path_relations(
            composition
                .decision()
                .conflict()
                .graph()
                .expect("approval declares beneficiary conflict"),
        ),
        vec!["EstateBeneficiary"]
    );
    assert_eq!(
        path_relations(
            composition
                .actors()
                .distinct_actor()
                .graph()
                .expect("approval declares requester separation"),
        ),
        vec!["CapabilityEstate", "EmergencyGrant", "EmergencyRequester"]
    );
    assert_eq!(
        anchor_relations(
            &composition
                .actors()
                .distinct_actor()
                .graph()
                .unwrap()
                .requirements()[0]
                .clauses()[0]
        ),
        vec!["EmergencyRequester"]
    );
    assert!(matches!(
        composition.propagation().disclosure(),
        ApplicationCapabilityDisclosureRule::NotApplicable
    ));
}

#[test]
fn mandatory_review_contract_installs_every_actor_exclusion() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            CompleteEstateMandatoryReviewCapability::reference(),
            CompleteEstateMandatoryReviewOperation::reference(),
        )
        .unwrap();
    let composition = capability.contract().composition();

    assert_eq!(
        path_relations(
            composition
                .decision()
                .conflict()
                .graph()
                .expect("review declares beneficiary conflict"),
        ),
        vec!["EstateBeneficiary"]
    );
    assert_eq!(
        path_relations(
            composition
                .actors()
                .separation_of_duty()
                .graph()
                .expect("review declares executor separation"),
        ),
        vec!["EstateExecutor"]
    );
    let distinct = composition
        .actors()
        .distinct_actor()
        .graph()
        .expect("review declares requester and approver separation");
    assert_eq!(requirement_widths(distinct), vec![2]);
    let mut anchors = all_clauses(distinct)
        .iter()
        .map(|clause| anchor_relations(clause))
        .collect::<Vec<_>>();
    anchors.sort();
    assert_eq!(
        anchors,
        vec![
            vec!["EmergencyApprover", "EmergencyReview"],
            vec!["EmergencyRequester", "EmergencyReview"],
        ]
    );
    assert_eq!(
        path_relations(distinct),
        vec![
            "EmergencyApprover",
            "EmergencyRequester",
            "EmergencyReview",
            "ReviewEstate"
        ]
    );
    let allow = composition.decision().allow().graph();
    assert_eq!(requirement_widths(allow), vec![3]);
    assert!(all_clauses(allow)
        .iter()
        .all(|clause| clause.context_anchors().is_empty()));
}

#[test]
fn transition_output_actors_are_not_required_as_prestate() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let recognition = bank
        .capability(
            RecognizeEstateExecutorCapability::reference(),
            RecognizeEstateExecutorOperation::reference(),
        )
        .unwrap();
    let separation = recognition
        .contract()
        .composition()
        .actors()
        .separation_of_duty()
        .graph()
        .expect("executor recognition declares exact legal-authority separation");
    assert_eq!(
        path_relations(separation),
        vec!["LegalAuthorityEstate", "LegalAuthorityHolder"]
    );
    assert_eq!(
        all_clauses(separation)
            .iter()
            .flat_map(|clause| anchor_relations(clause))
            .collect::<Vec<_>>(),
        vec!["LegalAuthorityHolder"]
    );

    let request = bank
        .capability(
            RequestEstateEmergencyAccessCapability::reference(),
            RequestEstateEmergencyAccessOperation::reference(),
        )
        .unwrap();
    let allow = request.contract().composition().decision().allow().graph();
    assert_eq!(requirement_widths(allow), vec![4]);
    assert!(all_clauses(allow)
        .iter()
        .all(|clause| clause.context_anchors().is_empty()));
}

#[test]
fn estate_contract_installs_exact_grant_currentness_meaning() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            RecognizeEstateExecutorCapability::reference(),
            RecognizeEstateExecutorOperation::reference(),
        )
        .unwrap();
    let currentness = capability.contract().constraints().currentness();

    assert_eq!(
        currentness.active_status().field().field(),
        "CapabilityGrantStatusField"
    );
    assert_eq!(
        currentness.active_status().value(),
        &CapabilityGrantStatusBinding::encode(&CapabilityGrantStatus::Active).unwrap()
    );
    assert_eq!(
        currentness.validity().timeline(),
        worth_query_decl::facade::application_capability::
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds
    );
    assert_eq!(
        currentness.workflow().grant().field(),
        "CapabilityWorkflowStageField"
    );
    assert_eq!(
        currentness.workflow().resource().field(),
        "EstateWorkflowStageField"
    );
    assert_eq!(
        currentness.validity().not_before().field(),
        "CapabilityValidFromField"
    );
    assert_eq!(
        currentness.validity().not_after().field(),
        "CapabilityValidThroughField"
    );
}

fn path_relations(rule: &ApplicationCapabilityGraphRule) -> Vec<&str> {
    let mut relations = all_clauses(rule)
        .iter()
        .flat_map(|clause| clause.path().traversals())
        .map(|traversal| traversal.relation())
        .collect::<Vec<_>>();
    relations.sort_unstable();
    relations.dedup();
    relations
}

fn all_clauses(
    rule: &ApplicationCapabilityGraphRule,
) -> Vec<&worth_query_host::facade::declaration::application_capability::ApplicationCapabilityGraphClause>
{
    rule.requirements()
        .iter()
        .flat_map(|requirement| requirement.clauses())
        .collect()
}

fn requirement_widths(rule: &ApplicationCapabilityGraphRule) -> Vec<usize> {
    let mut widths = rule
        .requirements()
        .iter()
        .map(|requirement| requirement.clauses().len())
        .collect::<Vec<_>>();
    widths.sort_unstable();
    widths
}

fn anchor_relations(
    clause: &worth_query_host::facade::declaration::application_capability::ApplicationCapabilityGraphClause,
) -> Vec<&str> {
    clause
        .context_anchors()
        .iter()
        .map(|anchor| anchor.relation().relation())
        .collect()
}

fn expected_values(
    fields: impl IntoIterator<Item = RestrictedBankField>,
) -> Vec<worth_foundational::facade::AspectValue> {
    let mut values = fields
        .into_iter()
        .map(|field| RestrictedBankFieldBinding::encode(&field).unwrap())
        .collect::<Vec<_>>();
    values.sort();
    values
}

fn expected_roles(
    roles: impl IntoIterator<Item = EmployeeRole>,
) -> Vec<worth_foundational::facade::AspectValue> {
    let mut values = roles
        .into_iter()
        .map(|role| EmployeeRoleBinding::encode(&role).unwrap())
        .collect::<Vec<_>>();
    values.sort();
    values
}
