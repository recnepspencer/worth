pub use std::time::{Duration, Instant, SystemTime};

pub use bank_domain::{
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
pub use bank_server::{
    BankAuthenticatedPrincipal, BankAuthenticationConfiguration, BankEmployeeAssignmentSeed,
    BankIdentityRuntime, BankPrincipalSeed, BankReadControls, BankWorldSeed,
};
pub use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
        WorthQueryCancellationSource, WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
    },
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
};

worth_query_host::facade::worth_query_authentication_audience!(BankEstateCertificationAudience);
worth_query_host::facade::worth_query_principal_issuer!(EstateCertificationBankTestInvalidIssuer);
worth_query_host::facade::worth_query_authentication_method!(ExternalCertificationAdapterMethod);

pub const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub const BRANCH: BranchId = BranchId::new(2).unwrap();
pub const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub const ACCOUNT: AccountId = AccountId::new(4).unwrap();
pub const DECEASED: BankPrincipalId = BankPrincipalId::new(5).unwrap();
pub const EXECUTOR: BankPrincipalId = BankPrincipalId::new(6).unwrap();
pub const REVIEWER: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(8).unwrap();

pub struct CertificationFixture {
    pub runtime: BankIdentityRuntime,
    pub principal: BankAuthenticatedPrincipal,
    pub controls: BankReadControls,
}

pub fn certification_fixture() -> CertificationFixture {
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
        .admit_authentication_adapter(authentication_configuration(), local_principal(identity))
        .expect("the public authentication boundary installs");
    let request = request_scope();
    let principal = runtime
        .authenticate_local(&authentication, &request)
        .expect("the mapped compliance principal authenticates");
    let controls = BankReadControls::current(request, 1).unwrap();
    CertificationFixture {
        runtime,
        principal,
        controls,
    }
}

pub fn snapshot() -> bank_domain::proposals::BankSnapshot {
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

pub fn estate_world() -> BankEstateWorld {
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

pub fn legal_compliance_grant() -> EstateCapabilityGrant {
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

pub fn external_identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(EstateCertificationBankTestInvalidIssuer, subject)
        .unwrap()
}

pub fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}

pub fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::of(BankEstateCertificationAudience),
        WorthQueryAuthenticationMethod::of(ExternalCertificationAdapterMethod),
    )
}

/// The Host's local process principal acting as the mapped compliance reviewer.
pub fn local_principal(
    identity: WorthQueryExternalPrincipalIdentity,
) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        identity,
        BankEstateCertificationAudience,
        ExternalCertificationAdapterMethod,
        Duration::from_secs(60),
    )
    .unwrap()
}
