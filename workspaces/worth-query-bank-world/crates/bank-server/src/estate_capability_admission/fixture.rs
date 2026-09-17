#[path = "fixture/authentication.rs"]
mod authentication;
#[path = "fixture/authorization_time.rs"]
mod authorization_time;
#[path = "fixture/capability_fixture.rs"]
mod capability_fixture;
#[path = "fixture/estate_baseline.rs"]
mod estate_baseline;
#[path = "fixture/grant_spec.rs"]
mod grant_spec;
#[path = "fixture/identity_catalog.rs"]
mod identity_catalog;
#[path = "fixture/world.rs"]
mod world;

pub(crate) use authentication::request_scope;
use authentication::{
    authentication_configuration, block_on, external_identity, TestAuthenticationAdapter,
    TestCredential,
};
pub(crate) use authorization_time::AuthorizationTimeController;
use bank_domain::{estate::EstateWorkflowStage, model::BankPrincipalId};
pub(crate) use capability_fixture::CapabilityFixture;
use estate_baseline::{base_estate, extra_principal, grant};
pub(crate) use grant_spec::GrantSpec;
pub(crate) use identity_catalog::*;
pub(crate) use world::{
    foreign_estate_revocation_world, WarmLocalityAxis, FOREIGN_ESTATE, FOREIGN_GRANT,
};
use world::{install_fixture_world, FixtureWorldComposition, FixtureWorldSpec};

pub(super) fn capability_world(
    scenario: &str,
    spec: GrantSpec,
    case_stage: EstateWorkflowStage,
) -> CapabilityFixture {
    admission_world(AdmissionWorldSpec {
        scenario,
        grant: spec,
        case_stage,
        specialist_holds_authority: false,
        unrelated_grants: 0,
    })
}

pub(super) fn capability_world_with_specialist_authority(
    scenario: &str,
    spec: GrantSpec,
    case_stage: EstateWorkflowStage,
) -> CapabilityFixture {
    admission_world(AdmissionWorldSpec {
        scenario,
        grant: spec,
        case_stage,
        specialist_holds_authority: true,
        unrelated_grants: 0,
    })
}

pub(super) fn capability_world_with_unrelated_grants(
    scenario: &str,
    spec: GrantSpec,
    case_stage: EstateWorkflowStage,
    unrelated_grants: usize,
) -> CapabilityFixture {
    admission_world(AdmissionWorldSpec {
        scenario,
        grant: spec,
        case_stage,
        specialist_holds_authority: false,
        unrelated_grants,
    })
}

struct AdmissionWorldSpec<'a> {
    scenario: &'a str,
    grant: GrantSpec,
    case_stage: EstateWorkflowStage,
    specialist_holds_authority: bool,
    unrelated_grants: usize,
}

fn admission_world(spec: AdmissionWorldSpec<'_>) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario: spec.scenario,
        spec: spec.grant,
        case_stage: spec.case_stage,
        specialist_holds_authority: spec.specialist_holds_authority,
        unrelated_grants: spec.unrelated_grants,
        composition: FixtureWorldComposition::Admission,
        alternate_emergency_bound: None,
    })
}

pub(super) fn governed_disclosure_world(
    scenario: &str,
    beneficiary: BankPrincipalId,
) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::identity_verification(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Disclosure { beneficiary },
        alternate_emergency_bound: None,
    })
}

pub(super) fn emergency_request_world(
    scenario: &str,
    upper_bound: GrantSpec,
    case_stage: EstateWorkflowStage,
) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: upper_bound,
        case_stage,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Lifecycle,
        alternate_emergency_bound: None,
    })
}

pub(super) fn emergency_request_world_at(
    scenario: &str,
    upper_bound: GrantSpec,
    case_stage: EstateWorkflowStage,
    authorization_time: AuthorizationTimeController,
) -> CapabilityFixture {
    capability_world_from_spec_with_authorization_time(
        FixtureWorldSpec {
            scenario,
            spec: upper_bound,
            case_stage,
            specialist_holds_authority: false,
            unrelated_grants: 0,
            composition: FixtureWorldComposition::Lifecycle,
            alternate_emergency_bound: None,
        },
        authorization_time,
    )
}

pub(crate) fn emergency_request_world_with_alternate_bound(
    scenario: &str,
    upper_bound: GrantSpec,
    alternate_bound: GrantSpec,
    case_stage: EstateWorkflowStage,
) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: upper_bound,
        case_stage,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Lifecycle,
        alternate_emergency_bound: Some(alternate_bound),
    })
}

pub(super) fn governance_projection_world(scenario: &str) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::governance_view(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::GovernanceProjection,
        alternate_emergency_bound: None,
    })
}

pub(super) fn product_projection_world(scenario: &str, grant: GrantSpec) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: grant,
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::ProductProjection,
        alternate_emergency_bound: None,
    })
}

pub(super) fn warm_locality_world(
    scenario: &str,
    axis: WarmLocalityAxis,
    count: usize,
) -> CapabilityFixture {
    let unrelated_grants = if matches!(axis, WarmLocalityAxis::Grants) {
        count
    } else {
        0
    };
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::legal_compliance_view(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants,
        composition: FixtureWorldComposition::WarmLocality { axis, count },
        alternate_emergency_bound: None,
    })
}

pub(crate) fn delegation_world(scenario: &str) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::governance_view(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Delegation {
            command_authority: true,
            parent_context: world::DelegationParentContext::Exact,
        },
        alternate_emergency_bound: None,
    })
}

pub(super) fn delegation_world_without_command(scenario: &str) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::governance_view(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Delegation {
            command_authority: false,
            parent_context: world::DelegationParentContext::Exact,
        },
        alternate_emergency_bound: None,
    })
}

pub(super) fn delegation_world_with_parent_branch_mismatch(scenario: &str) -> CapabilityFixture {
    delegation_world_with_parent_context(scenario, world::DelegationParentContext::Branch)
}

pub(super) fn delegation_world_with_parent_institution_mismatch(
    scenario: &str,
) -> CapabilityFixture {
    delegation_world_with_parent_context(scenario, world::DelegationParentContext::Institution)
}

fn delegation_world_with_parent_context(
    scenario: &str,
    parent_context: world::DelegationParentContext,
) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: GrantSpec::governance_view(),
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Delegation {
            command_authority: true,
            parent_context,
        },
        alternate_emergency_bound: None,
    })
}

pub(crate) fn delegation_world_with_parent_spec(
    scenario: &str,
    parent: GrantSpec,
) -> CapabilityFixture {
    capability_world_from_spec(FixtureWorldSpec {
        scenario,
        spec: parent,
        case_stage: EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Delegation {
            command_authority: true,
            parent_context: world::DelegationParentContext::Exact,
        },
        alternate_emergency_bound: None,
    })
}

pub(crate) fn delegation_world_with_parent_spec_at(
    scenario: &str,
    parent: GrantSpec,
    authorization_time: AuthorizationTimeController,
) -> CapabilityFixture {
    capability_world_from_spec_with_authorization_time(
        FixtureWorldSpec {
            scenario,
            spec: parent,
            case_stage: EstateWorkflowStage::Administration,
            specialist_holds_authority: false,
            unrelated_grants: 0,
            composition: FixtureWorldComposition::Delegation {
                command_authority: true,
                parent_context: world::DelegationParentContext::Exact,
            },
            alternate_emergency_bound: None,
        },
        authorization_time,
    )
}

fn capability_world_from_spec(spec: FixtureWorldSpec<'_>) -> CapabilityFixture {
    let installed = install_fixture_world(spec);
    capability_fixture(installed)
}

fn capability_world_from_spec_with_authorization_time(
    spec: FixtureWorldSpec<'_>,
    authorization_time: AuthorizationTimeController,
) -> CapabilityFixture {
    let installed = world::install_fixture_world_with_authorization_time(spec, authorization_time);
    capability_fixture(installed)
}

fn capability_fixture(installed: world::InstalledFixtureWorld) -> CapabilityFixture {
    let runtime = installed.runtime;
    let authentication = runtime
        .admit_authentication_adapter(authentication_configuration(), TestAuthenticationAdapter)
        .expect("the causal test adapter should install");
    CapabilityFixture {
        runtime,
        estate_world: installed.estate_world,
        authentication,
        specialist_identity: installed.identities[1].clone(),
        executor_identity: installed.identities[2].clone(),
        approver_identity: installed.identities[3].clone(),
        reviewer_identity: installed.identities[4].clone(),
    }
}
