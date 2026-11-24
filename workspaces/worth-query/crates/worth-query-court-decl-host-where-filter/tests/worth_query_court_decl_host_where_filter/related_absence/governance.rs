//! Governance and declaration-boundary courts for related-record absence
//! predicates: unauthorized relation or target-field reads fail closed before
//! touching rows or edges, even on empty projects.
mod schema;
use schema::*;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial,
};
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};
use worth_query_testkit_document_retention::facade::{
    document_retention_model::operator_identity::{
        authenticate_operator, operator_identity, request_scope,
    },
    document_world_resources::world_resources,
};

struct GovProgram;
struct GovFeature;
impl ApplicationFeature<GovSchema> for GovFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "related-absence.gov-feature.v1";
}
impl ApplicationProgramDefinition<GovSchema> for GovProgram {
    type Contributions = (GovContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("related-absence.gov-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<GovSchema, GovFeature>().finish()]
    }
}
impl WorthQueryApplicationContribution<GovSchema> for GovContribution {
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut WorthQueryApplicationContributionSetup<'_, GovSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }
}

fn key<E>(k: &str) -> WorthQueryApplicationEntityKey<GovSchema, E> {
    WorthQueryApplicationEntityKey::new(k.to_owned()).unwrap()
}

fn install_governed() -> WorthQueryProgramApplicationRuntime<GovSchema, GovProgram> {
    in_memory_program(
        ApplicationProgramAuthoring::<GovSchema, GovProgram>::begin()
            .validated_program()
            .unwrap(),
        GovSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("79797979-7979-7979-7979-797979797979").unwrap(),
        WorthQueryApplicationLimits::new(
            world_resources(
                128,
                64 * 1024 * 1024,
                256,
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
            WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(64).unwrap()),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(GovPrincipalBinding::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("principal-1").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for proj in ["p1", "empty"] {
                let pstr = format!("project-{proj}");
                let gstr = format!("grant-{proj}");
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Project::reference(), key(&pstr))
                        .field(ProjectKey::at("ProjectKey"), proj.to_owned()),
                )?;
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(BoardGrant::reference(), key(&gstr))
                        .field(GrantIdentity::at("GrantIdentity"), gstr.clone())
                        .field(GrantAction::at("GrantAction"), 1_u64)
                        .field(GrantPurpose::at("GrantPurpose"), 1_u64)
                        .field(GrantStatus::at("GrantStatus"), 1_u64)
                        .field(GrantScope::at("GrantScope"), proj.to_owned())
                        .field(GrantNotBefore::at("GrantNotBefore"), 0_u64)
                        .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX / 2)
                        .field(GrantDepth::at("GrantDepth"), 0_u64)
                        .field(GrantDisclosure::at("GrantDisclosure"), 1_u64),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    GrantGrantee::reference(),
                    format!("grantee-{proj}"),
                    key("principal-1"),
                    key(&gstr),
                ))?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    GrantGrantor::reference(),
                    format!("grantor-{proj}"),
                    key("principal-1"),
                    key(&gstr),
                ))?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    GrantResource::reference(),
                    format!("resource-{proj}"),
                    key(&gstr),
                    key(&pstr),
                ))?;
            }
            for (tid, status) in [(1_u64, "open"), (2, "open"), (3, "open")] {
                let tstr = format!("task-{tid}");
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Task::reference(), key(&tstr))
                        .field(TaskKey::at("TaskKey"), tid)
                        .field(Status::at("Status"), status.to_owned()),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Contains::reference(),
                    format!("contains-{tid}"),
                    key("project-p1"),
                    key(&tstr),
                ))?;
            }
            // Task 2 is blocked by open Task 3 -> only Tasks 1 and 3 are Ready!
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                BlockedBy::reference(),
                "blocked-2-3".to_owned(),
                key("task-2"),
                key("task-3"),
            ))?;
            Ok(())
        },
    )
    .expect("governed board installs")
}

#[test]
fn governed_related_absence_enforces_relation_and_target_field_disclosure() {
    let host = install_governed();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let access = |project: &str| ReadBoardAccess {
        project: project.to_owned(),
    };

    // 1. Authorized principal with BOTH relation (tag 1) and Status (tag 1) disclosure succeeds.
    let ok = host
        .runtime()
        .request(&principal, &scope)
        .query(GovAllowedIntent {
            project: "p1".into(),
        })
        .execute_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            access("p1"),
        )
        .expect("authorized governed related-absence query succeeds");
    assert_eq!(ok.rows().iter().map(|r| r.key).collect::<Vec<_>>(), [1, 3]);

    // 2. Omitting `use_relation_by` refuses at disclosure contract compilation.
    let missing_rule = host
        .runtime()
        .request(&principal, &scope)
        .query(GovMissingRelIntent {
            project: "p1".into(),
        })
        .execute_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            access("p1"),
        )
        .err()
        .expect("missing internal relation disclosure rule must refuse");
    assert!(
        matches!(
            missing_rule,
            WorthQueryApplicationRequestQueryDenial::Admission(ref cause)
                if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
        ),
        "unexpected denial: {missing_rule:?}"
    );

    // 3. Unauthorized relation disclosure (requires tag 2, principal has tag 1) refuses fail-closed
    //    on BOTH populated project AND empty project (before touching rows or edges!).
    for proj in ["p1", "empty"] {
        let denied_rel = host
            .runtime()
            .request(&principal, &scope)
            .query(GovDeniedRelIntent {
                project: proj.into(),
            })
            .execute_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                access(proj),
            )
            .err()
            .expect("unauthorized relation disclosure must refuse");
        assert!(
            matches!(
                denied_rel,
                WorthQueryApplicationRequestQueryDenial::Execution(_)
                    | WorthQueryApplicationRequestQueryDenial::Admission(_)
            ),
            "unexpected denial on {proj}: {denied_rel:?}"
        );
    }

    // 4. Unauthorized target field disclosure (relation tag 1 allowed, Status requires tag 2) refuses!
    let denied_fld = host
        .runtime()
        .request(&principal, &scope)
        .query(GovDeniedFieldIntent {
            project: "p1".into(),
        })
        .execute_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            access("p1"),
        )
        .err()
        .expect("unauthorized target-field disclosure must refuse");
    assert!(
        matches!(
            denied_fld,
            WorthQueryApplicationRequestQueryDenial::Execution(_)
                | WorthQueryApplicationRequestQueryDenial::Admission(_)
        ),
        "unexpected denial: {denied_fld:?}"
    );
}
