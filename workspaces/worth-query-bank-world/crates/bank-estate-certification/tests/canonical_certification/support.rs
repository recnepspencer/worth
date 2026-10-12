use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, Instant, SystemTime},
};

use bank_domain::{
    estate::{
        BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
        DelegationLimit, EstateBranch, EstateCapabilityGrant, EstateCapabilityOperation,
        EstateCapabilityPurpose, EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus,
        EstateDeathNotice, EstateEmployeeAssignment, EstateLegalAuthority, EstateMoment,
        EstateWorkflowStage, LegalAuthorityId, LegalAuthorityKind, RestrictedBankField,
    },
    model::{
        AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
        EmployeeRole, InstitutionId,
    },
    proposals::BankSnapshotBuilder,
    schema::AccountStatus,
};
use bank_server::{
    BankAuthenticatedPrincipal, BankAuthenticationConfiguration, BankEmployeeAssignmentSeed,
    BankIdentityRuntime, BankPrincipalSeed, BankReadControls, BankWorldSeed,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterFailure,
        WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
        WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod,
        WorthQueryCancellationSource, WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
    },
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
};

const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
const BRANCH: BranchId = BranchId::new(2).unwrap();
pub(super) const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
const ACCOUNT: AccountId = AccountId::new(4).unwrap();
const DECEASED: BankPrincipalId = BankPrincipalId::new(5).unwrap();
const EXECUTOR: BankPrincipalId = BankPrincipalId::new(6).unwrap();
const REVIEWER: BankPrincipalId = BankPrincipalId::new(7).unwrap();
const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(8).unwrap();

pub(super) struct CertificationFixture {
    pub(super) runtime: BankIdentityRuntime,
    pub(super) principal: BankAuthenticatedPrincipal,
    pub(super) controls: BankReadControls,
}

pub(super) fn certification_fixture() -> CertificationFixture {
    let deceased_identity = external_identity("deceased");
    let executor_identity = external_identity("executor");
    let identity = external_identity("compliance-reviewer");
    let seed = BankWorldSeed::new(snapshot())
        .principal(BankPrincipalSeed::enabled(DECEASED, deceased_identity))
        .principal(BankPrincipalSeed::enabled(EXECUTOR, executor_identity))
        .principal(BankPrincipalSeed::enabled(REVIEWER, identity.clone()))
        .employee(BankEmployeeAssignmentSeed::new(
            ASSIGNMENT,
            INSTITUTION,
            REVIEWER,
            EmployeeRole::Compliance,
        ))
        .estate(estate_world());
    let runtime = BankIdentityRuntime::install_world(seed).expect("the public Bank world installs");
    let authentication = runtime
        .admit_authentication_adapter(authentication_configuration(), TestAdapter)
        .expect("the public authentication boundary installs");
    let request = request_scope();
    let principal =
        block_on(runtime.authenticate_with(&authentication, TestCredential(identity), &request))
            .expect("the mapped compliance principal authenticates");
    let controls = BankReadControls::current(request, 1, 20_000).unwrap();
    CertificationFixture {
        runtime,
        principal,
        controls,
    }
}

fn snapshot() -> bank_domain::proposals::BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(EXECUTOR)
        .principal(REVIEWER)
        .personal_account(
            ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Certified Estate").unwrap(),
            AccountStatus::Frozen,
        )
        .build()
        .expect("the external fixture snapshot is valid")
}

fn estate_world() -> BankEstateWorld {
    BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: bank_domain::estate::DeathNoticeId::new(9).unwrap(),
            subject: DECEASED,
            status: bank_domain::estate::DeathNoticeStatus::Verified,
        })
        .with_case(EstateCase {
            id: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            deceased: DECEASED,
            account: ACCOUNT,
            death_notice: bank_domain::estate::DeathNoticeId::new(9).unwrap(),
            stage: EstateWorkflowStage::Administration,
            status: EstateCaseStatus::Open,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: REVIEWER,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::Compliance,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_legal_authority(EstateLegalAuthority {
            id: LegalAuthorityId::new(10).unwrap(),
            estate: ESTATE,
            holder: EXECUTOR,
            kind: LegalAuthorityKind::CourtAppointment,
            recognized: true,
        })
        .with_grant(legal_compliance_grant())
}

fn legal_compliance_grant() -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id: CapabilityGrantId::new(11).unwrap(),
        grantor: DECEASED,
        grantee: REVIEWER,
        scope: EstateCapabilityScope {
            account: None,
            estate: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::ViewRestrictedEstate,
            purpose: EstateCapabilityPurpose::LegalCompliance,
            field: Some(RestrictedBankField::LegalDocument),
            amount_ceiling: None,
            validity: CapabilityValidity::new(
                EstateMoment::from_epoch_seconds(0),
                EstateMoment::from_epoch_seconds(u64::MAX),
            )
            .unwrap(),
            delegation: DelegationLimit::none(),
            workflow_stage: EstateWorkflowStage::Administration,
        },
        parent: None,
        status: CapabilityGrantStatus::Active,
    }
}

fn external_identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        "https://estate-certification.bank.test.invalid",
        subject,
    )
    .unwrap()
}

fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(),
    )
}

fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::new("bank-estate-certification").unwrap(),
        WorthQueryAuthenticationMethod::new("external-certification-adapter").unwrap(),
    )
}

struct TestCredential(WorthQueryExternalPrincipalIdentity);
struct TestAdapter;

impl WorthQueryAuthenticationAdapter for TestAdapter {
    type Credential = TestCredential;

    fn configuration_identity(&self) -> &str {
        "bank-estate-external-certification-v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                credential.0,
                WorthQueryAuthenticationAudience::new("bank-estate-certification").unwrap(),
                WorthQueryAuthenticationMethod::new("external-certification-adapter").unwrap(),
                now,
                now + Duration::from_secs(60),
                Vec::new(),
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
