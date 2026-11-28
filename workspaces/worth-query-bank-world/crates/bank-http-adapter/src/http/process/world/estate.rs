use bank_domain::estate::{
    BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
    DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateBranch, EstateCapabilityGrant,
    EstateCapabilityOperation, EstateCapabilityPurpose, EstateCapabilityScope, EstateCase,
    EstateCaseId, EstateCaseStatus, EstateDeathNotice, EstateEmployeeAssignment,
    EstateLegalAuthority, EstateMoment, EstateWorkflowStage, LegalAuthorityId, LegalAuthorityKind,
};
use bank_domain::model::{
    AccountId, BankPrincipalId, EmployeeAssignmentId, EmployeeRole, InstitutionId, Money, USD,
};
use bank_server::BankEmployeeAssignmentSeed;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BankHttpProcessEstateWorld {
    pub branch: u64,
    pub estate: u64,
    pub estate_account: String,
    pub deceased_principal: u64,
    pub specialist_principal: u64,
    pub assignment: u64,
    pub notice: u64,
    pub grant: u64,
    pub aftermath: Option<BankHttpProcessEstateAftermathWorld>,
    pub elevation: Option<BankHttpProcessEstateElevationWorld>,
}

#[derive(Deserialize)]
pub struct BankHttpProcessEstateAftermathWorld {
    pub destination_account: String,
    pub beneficiary_principal: u64,
    pub executor_principal: u64,
    pub legal_authority: u64,
    pub disbursement_grant: u64,
    pub compensation_service_assignment: u64,
    pub amount_ceiling_minor: i64,
}

#[derive(Deserialize)]
pub struct BankHttpProcessEstateElevationWorld {
    pub requester_principal: u64,
    pub approver_principal: u64,
    pub reviewer_principal: u64,
    pub approver_assignment: u64,
    pub reviewer_assignment: u64,
    pub request_grant: u64,
    pub upper_bound_grant: u64,
    pub self_approval_grant: u64,
    pub approval_grant: u64,
    pub close_grant: u64,
    pub review_grant: u64,
}

pub(super) struct InstalledProcessEstate {
    pub(super) world: BankEstateWorld,
    pub(super) employees: Vec<BankEmployeeAssignmentSeed>,
}

impl BankHttpProcessEstateWorld {
    pub(super) fn build(self, institution: InstitutionId) -> Result<InstalledProcessEstate, ()> {
        let branch = BranchId::new(self.branch).ok_or(())?;
        let estate = EstateCaseId::new(self.estate).ok_or(())?;
        let account = AccountId::parse_canonical_text(&self.estate_account).ok_or(())?;
        let deceased = BankPrincipalId::new(self.deceased_principal).ok_or(())?;
        let specialist = BankPrincipalId::new(self.specialist_principal).ok_or(())?;
        let assignment = EmployeeAssignmentId::new(self.assignment).ok_or(())?;
        let notice = DeathNoticeId::new(self.notice).ok_or(())?;
        let mut world = BankEstateWorld::default()
            .with_branch(EstateBranch {
                id: branch,
                institution,
            })
            .with_death_notice(EstateDeathNotice {
                id: notice,
                subject: deceased,
                status: DeathNoticeStatus::Reported,
            })
            .with_case(EstateCase {
                id: estate,
                institution,
                branch,
                deceased,
                account,
                death_notice: notice,
                stage: EstateWorkflowStage::Administration,
                status: EstateCaseStatus::Open,
            })
            .with_assignment(EstateEmployeeAssignment {
                id: assignment,
                principal: specialist,
                institution,
                branch,
                role: EmployeeRole::EstateSpecialist,
            })
            .with_estate_assignment(estate, assignment)
            .with_grant(EstateCapabilityGrant {
                id: CapabilityGrantId::new(self.grant).ok_or(())?,
                grantor: deceased,
                grantee: specialist,
                scope: capability_scope(
                    institution,
                    branch,
                    estate,
                    EstateCapabilityOperation::NotifyDeath,
                    EstateCapabilityPurpose::EstateAdministration,
                    None,
                    None,
                )?,
                parent: None,
                status: CapabilityGrantStatus::Active,
            });
        let mut employees = vec![BankEmployeeAssignmentSeed::new(
            assignment,
            institution,
            specialist,
            EmployeeRole::EstateSpecialist,
        )];
        if let Some(aftermath) = self.aftermath {
            let installed = aftermath.install(
                world,
                EstateAftermathContext {
                    institution,
                    branch,
                    estate,
                    source_account: account,
                    grantor: deceased,
                    specialist,
                },
            )?;
            world = installed.world;
            employees.push(installed.compensation_service_employee);
        }
        if let Some(elevation) = self.elevation {
            let installed = elevation.install(
                world,
                EstateElevationContext {
                    institution,
                    branch,
                    estate,
                    grantor: deceased,
                },
            )?;
            world = installed.world;
            employees.extend(installed.employees);
        }
        Ok(InstalledProcessEstate { world, employees })
    }
}

struct EstateElevationContext {
    institution: InstitutionId,
    branch: BranchId,
    estate: EstateCaseId,
    grantor: BankPrincipalId,
}

struct InstalledEstateElevation {
    world: BankEstateWorld,
    employees: [BankEmployeeAssignmentSeed; 2],
}

impl BankHttpProcessEstateElevationWorld {
    fn install(
        self,
        world: BankEstateWorld,
        context: EstateElevationContext,
    ) -> Result<InstalledEstateElevation, ()> {
        let requester = BankPrincipalId::new(self.requester_principal).ok_or(())?;
        let approver = BankPrincipalId::new(self.approver_principal).ok_or(())?;
        let reviewer = BankPrincipalId::new(self.reviewer_principal).ok_or(())?;
        let approver_assignment = EmployeeAssignmentId::new(self.approver_assignment).ok_or(())?;
        let reviewer_assignment = EmployeeAssignmentId::new(self.reviewer_assignment).ok_or(())?;
        let world = world
            .with_assignment(EstateEmployeeAssignment {
                id: approver_assignment,
                principal: approver,
                institution: context.institution,
                branch: context.branch,
                role: EmployeeRole::EstateSpecialist,
            })
            .with_estate_assignment(context.estate, approver_assignment)
            .with_assignment(EstateEmployeeAssignment {
                id: reviewer_assignment,
                principal: reviewer,
                institution: context.institution,
                branch: context.branch,
                role: EmployeeRole::Compliance,
            })
            .with_estate_assignment(context.estate, reviewer_assignment);
        let grants = [
            (
                self.request_grant,
                requester,
                EstateCapabilityOperation::RequestEmergencyAccess,
                EstateCapabilityPurpose::EmergencyProtection,
                None,
            ),
            (
                self.upper_bound_grant,
                requester,
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::EmergencyProtection,
                Some(bank_domain::estate::RestrictedBankField::AccountDetails),
            ),
            (
                self.self_approval_grant,
                requester,
                EstateCapabilityOperation::ApproveEmergencyAccess,
                EstateCapabilityPurpose::EmergencyProtection,
                None,
            ),
            (
                self.approval_grant,
                approver,
                EstateCapabilityOperation::ApproveEmergencyAccess,
                EstateCapabilityPurpose::EmergencyProtection,
                None,
            ),
            (
                self.close_grant,
                approver,
                EstateCapabilityOperation::RevokeEmergencyAccess,
                EstateCapabilityPurpose::EmergencyProtection,
                None,
            ),
            (
                self.review_grant,
                reviewer,
                EstateCapabilityOperation::CompleteMandatoryReview,
                EstateCapabilityPurpose::MandatoryReview,
                None,
            ),
        ];
        let world = grants.into_iter().try_fold(
            world,
            |world, (id, grantee, operation, purpose, field)| {
                Ok::<_, ()>(world.with_grant(EstateCapabilityGrant {
                    id: CapabilityGrantId::new(id).ok_or(())?,
                    grantor: context.grantor,
                    grantee,
                    scope: EstateCapabilityScope {
                        field,
                        ..capability_scope(
                            context.institution,
                            context.branch,
                            context.estate,
                            operation,
                            purpose,
                            None,
                            None,
                        )?
                    },
                    parent: None,
                    status: CapabilityGrantStatus::Active,
                }))
            },
        )?;
        Ok(InstalledEstateElevation {
            world,
            employees: [
                BankEmployeeAssignmentSeed::new(
                    approver_assignment,
                    context.institution,
                    approver,
                    EmployeeRole::EstateSpecialist,
                ),
                BankEmployeeAssignmentSeed::new(
                    reviewer_assignment,
                    context.institution,
                    reviewer,
                    EmployeeRole::Compliance,
                ),
            ],
        })
    }
}

struct EstateAftermathContext {
    institution: InstitutionId,
    branch: BranchId,
    estate: EstateCaseId,
    source_account: AccountId,
    grantor: BankPrincipalId,
    specialist: BankPrincipalId,
}

struct InstalledEstateAftermath {
    world: BankEstateWorld,
    compensation_service_employee: BankEmployeeAssignmentSeed,
}

impl BankHttpProcessEstateAftermathWorld {
    fn install(
        self,
        world: BankEstateWorld,
        context: EstateAftermathContext,
    ) -> Result<InstalledEstateAftermath, ()> {
        let destination = AccountId::parse_canonical_text(&self.destination_account).ok_or(())?;
        let beneficiary = BankPrincipalId::new(self.beneficiary_principal).ok_or(())?;
        let executor = BankPrincipalId::new(self.executor_principal).ok_or(())?;
        let authority = LegalAuthorityId::new(self.legal_authority).ok_or(())?;
        let grant = CapabilityGrantId::new(self.disbursement_grant).ok_or(())?;
        let service_assignment =
            EmployeeAssignmentId::new(self.compensation_service_assignment).ok_or(())?;
        let amount_ceiling = Money::from_minor(self.amount_ceiling_minor).map_err(|_| ())?;
        let world = world
            .with_beneficiary(context.estate, beneficiary)
            .with_joint_owner(destination, beneficiary)
            .with_legal_authority(EstateLegalAuthority {
                id: authority,
                estate: context.estate,
                holder: executor,
                kind: LegalAuthorityKind::CourtAppointment,
                recognized: true,
            })
            .with_executor(context.estate, executor)
            .with_grant(EstateCapabilityGrant {
                id: grant,
                grantor: context.grantor,
                grantee: context.specialist,
                scope: capability_scope(
                    context.institution,
                    context.branch,
                    context.estate,
                    EstateCapabilityOperation::DisburseEstate,
                    EstateCapabilityPurpose::EstateDisbursement,
                    Some(context.source_account),
                    Some(amount_ceiling),
                )?,
                parent: None,
                status: CapabilityGrantStatus::Active,
            });
        Ok(InstalledEstateAftermath {
            world,
            compensation_service_employee: BankEmployeeAssignmentSeed::new(
                service_assignment,
                context.institution,
                context.specialist,
                EmployeeRole::Teller,
            ),
        })
    }
}

fn capability_scope(
    institution: InstitutionId,
    branch: BranchId,
    estate: EstateCaseId,
    operation: EstateCapabilityOperation,
    purpose: EstateCapabilityPurpose,
    account: Option<AccountId>,
    amount_ceiling: Option<Money<USD>>,
) -> Result<EstateCapabilityScope, ()> {
    Ok(EstateCapabilityScope {
        account,
        estate,
        institution,
        branch,
        operation,
        purpose,
        field: None,
        amount_ceiling,
        validity: CapabilityValidity::new(
            EstateMoment::from_epoch_seconds(0),
            EstateMoment::from_epoch_seconds(u64::MAX),
        )
        .ok_or(())?,
        delegation: DelegationLimit::none(),
        workflow_stage: EstateWorkflowStage::Administration,
    })
}
