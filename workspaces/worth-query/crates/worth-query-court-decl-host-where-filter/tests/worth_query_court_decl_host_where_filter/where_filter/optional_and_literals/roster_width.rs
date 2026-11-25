//! Proves a fixed single-field residual `where_filter` charges admitted work
//! proportional to the root kind's executable aspect binding roster width.
#[path = "roster_width/schema.rs"]
mod schema;
use schema::*;

use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_testkit_document_retention::facade::{
    document_retention_model::operator_identity::{authenticate_operator, operator_identity},
    document_world_resources::world_resources,
};

struct RosterProgram;
struct RosterFeature;
impl ApplicationFeature<RosterSchema> for RosterFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "roster.feature.v1";
}
impl ApplicationProgramDefinition<RosterSchema> for RosterProgram {
    type Contributions = (RosterContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new("roster.prog.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<RosterSchema, RosterFeature>().finish()]
    }
}
impl WorthQueryApplicationContribution<RosterSchema> for RosterContribution {
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut WorthQueryApplicationContributionSetup<'_, RosterSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }
}

macro_rules! ek {
    ($prefix:literal, $id:expr) => {
        WorthQueryApplicationEntityKey::new(format!("{}-{}", $prefix, $id)).unwrap()
    };
}

fn install_roster_host() -> WorthQueryProgramApplicationRuntime<RosterSchema, RosterProgram> {
    in_memory_program(
        ApplicationProgramAuthoring::<RosterSchema, RosterProgram>::begin()
            .validated_program()
            .unwrap(),
        <RosterSchema as ApplicationSchema>::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("57575757-5757-5757-5757-575757575758").unwrap(),
        WorthQueryApplicationLimits::new(
            world_resources(
                128,
                64 * 1024 * 1024,
                256,
                WorthQueryAdmissionClock::system(),
            ),
            WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(32).unwrap(),
            ),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(RosterPrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("roster-author").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(entity(RosterBoard, Group), ek!("group", 1))
                    .field(GroupKeyField::at("GroupKey"), 1_u64),
            )?;
            for id in 1..=16_u64 {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        entity(RosterBoard, NarrowItem),
                        ek!("narrow", id),
                    )
                    .field(NarrowKey::at("ItemKey"), id)
                    .field(NarrowTag::at("Tag"), "miss".to_owned()),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    relation(RosterBoard, Group, HasNarrow),
                    format!("has-narrow-{id}"),
                    ek!("group", 1),
                    ek!("narrow", id),
                ))?;
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        entity(RosterBoard, WideItem),
                        ek!("wide", id),
                    )
                    .field(WideKey::at("ItemKey"), id)
                    .field(WideTag::at("Tag"), "miss".to_owned()),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    relation(RosterBoard, Group, HasWide),
                    format!("has-wide-{id}"),
                    ek!("group", 1),
                    ek!("wide", id),
                ))?;
            }
            Ok(())
        },
    )
    .expect("roster width application installs")
}

macro_rules! run_funded {
    ($host:expr, $scope:expr, $work:expr, $intent:expr) => {{
        let caller = ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(64 * 1024 * 1024),
                    Some($work),
                ),
            ),
            cancellation: $scope.cancellation().execution_token(),
            deadline: $scope.deadline(),
        })
        .unwrap();
        let principal = authenticate_operator($host.installed_schema(), $scope);
        $host
            .runtime()
            .with_funded_application_advancement(
                $scope,
                ExecutionAllocationPolicy::Execution(&caller),
                |phase| {
                    $host
                        .runtime()
                        .request(&principal, $scope)
                        .query($intent)
                        .execute_in_advancement(&phase)
                        .map(|res| {
                            (
                                res.rows().to_vec(),
                                res.receipt().inspect().read_work().total_work_units(),
                            )
                        })
                },
            )
            .unwrap()
    }};
}

#[test]
fn fixed_single_field_where_filter_work_scales_with_root_kind_roster_width() {
    let host = install_roster_host();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    );
    let (narrow_rows, narrow_work) =
        run_funded!(&host, &scope, 1 << 24, RunNarrow { group: 1 }).unwrap();
    let (wide_rows, wide_work) = run_funded!(&host, &scope, 1 << 24, RunWide { group: 1 }).unwrap();
    assert!(narrow_rows.is_empty() && wide_rows.is_empty());
    // WideItem has 8 installed aspects vs NarrowItem's 1 (delta = 7); across 16
    // candidates, roster-width work must increase by at least 16 * 3 * 7 = 336.
    assert!(
        wide_work >= narrow_work + 16 * 3 * 7,
        "wider root-kind aspect roster must raise admitted residual work: narrow={narrow_work}, wide={wide_work}"
    );
}
