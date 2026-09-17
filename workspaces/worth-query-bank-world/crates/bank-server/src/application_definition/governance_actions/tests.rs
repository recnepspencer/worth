use bank_domain::model::BankPrincipalId;
use bank_domain::schema::{
    BankSchema, EstateActionInputBinding, RequestEstateEmergencyAccessOperation,
    ViewRestrictedEstateOperation,
};
use worth_query_host::facade::{
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
    declaration::{
        application_program::{
            ApplicationActionLeaf, ApplicationActionList, ApplicationNoOutputGraph,
            ApplicationOperationActionRef, ApplicationProgramAuthoring,
            ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
        },
        application_schema::ApplicationOperationMarkerIdentity,
    },
    domain::install_application_program,
    primary_graph::WorthQueryApplicationCommitDenialKind,
};

use super::super::{
    composition::{BankEstateFeature, BankFeatures, BankRules},
    providers::{BankAccountsProvider, BankEstateProvider, BankPaymentsProvider},
};
use crate::{BankIdentityRuntime, BankPrincipalSeed};

struct ForgedRequestOperation;

impl ApplicationOperationMarkerIdentity<BankSchema> for ForgedRequestOperation {
    type InputBinding = EstateActionInputBinding;
    const IDENTIFIER: &'static str = RequestEstateEmergencyAccessOperation::IDENTIFIER;
}

struct ForgedBankApplication;

impl ApplicationProgramDefinition<BankSchema> for ForgedBankApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Actions = ApplicationActionList<
        ApplicationOperationActionRef<BankSchema, BankEstateFeature, ForgedRequestOperation>,
        ApplicationActionLeaf,
    >;
    type Features = BankFeatures;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.forged-request-test.v1");
}

#[test]
fn specialized_action_requires_exact_installed_operation_provenance() {
    let runtime = installed_bank_program();
    let program = ApplicationProgramAuthoring::<BankSchema, ForgedBankApplication>::begin()
        .validated_program()
        .expect("the forged marker has valid shape so installation must make the decision");
    let denial = match install_application_program(
        program,
        runtime.application_runtime().installed_schema(),
    ) {
        Ok(_) => panic!("a marker sharing the operation name must not gain installed authority"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.subject(),
        RequestEstateEmergencyAccessOperation::IDENTIFIER
    );
}

#[test]
fn specialized_admission_refuses_an_unlisted_installed_operation() {
    let runtime = installed_bank_program();
    assert!(runtime
        .application_program()
        .admit_program_operation::<RequestEstateEmergencyAccessOperation>()
        .is_ok());
    let denial = match runtime
        .application_program()
        .admit_program_operation::<ViewRestrictedEstateOperation>()
    {
        Ok(_) => panic!("an installed operation outside BankActions must not enter the program"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
}

fn installed_bank_program() -> BankIdentityRuntime {
    BankIdentityRuntime::install([BankPrincipalSeed::enabled(
        BankPrincipalId::new(1).expect("one is a valid Bank principal identity"),
        WorthQueryExternalPrincipalIdentity::new("bank-program-test", "one")
            .expect("the test identity has valid external components"),
    )])
    .expect("the real Bank program installs with one principal binding")
}
