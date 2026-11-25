//! Governance courts for related-record absence predicates: unauthorized
//! relation OR unauthorized target-only field (`Gate`, never referenced on the
//! root!) fails closed before touching rows or edges, even on empty projects.
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
            WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(64).unwrap(),
            ),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(GovPrincipalBinding::reference())
                    .unwrap(),
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
            for (tid, status, gate) in [
                (1_u64, "open", "clear"),
                (2, "open", "clear"),
                (3, "open", "blocking"),
            ] {
                let tstr = format!("task-{tid}");
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Task::reference(), key(&tstr))
                        .field(TaskKey::at("TaskKey"), tid)
                        .field(Status::at("Status"), status.to_owned())
                        .field(Gate::at("Gate"), gate.to_owned()),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Contains::reference(),
                    format!("contains-{tid}"),
                    key("project-p1"),
                    key(&tstr),
                ))?;
            }
            // Task 2 is blocked by Task 3 whose target-only `Gate == "blocking"`.
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

fn is_internal_computation_denied<T>(
    result: Result<T, WorthQueryApplicationRequestQueryDenial>,
) -> bool {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::Admission(ref cause))
            if cause.kind()
                == WorthQueryApplicationQueryAdmissionDenialKind::InternalComputationDenied =>
        {
            true
        }
        Err(other) => panic!("expected Admission(InternalComputationDenied), got: {other:?}"),
        Ok(_) => panic!("expected Admission(InternalComputationDenied), got Ok"),
    }
}

/// Runs real governed denial probes against the Host runtime for TLA+ traces,
/// asserting exact typed `InternalComputationDenied` refusals.
pub(super) fn governed_trace_denial_probes() -> (bool, bool) {
    let host = install_governed();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let access = ReadBoardAccess {
        project: "p1".to_owned(),
    };
    let rel_denied = is_internal_computation_denied(
        host.runtime()
            .request(&principal, &scope)
            .query(GovDeniedRelIntent {
                project: "p1".into(),
            })
            .execute_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                access.clone(),
            ),
    );
    let field_denied = is_internal_computation_denied(
        host.runtime()
            .request(&principal, &scope)
            .query(GovDeniedFieldIntent {
                project: "p1".into(),
            })
            .execute_governed(ReadBoard::reference(), InspectBoard::reference(), access),
    );
    (rel_denied, field_denied)
}

#[test]
fn governed_related_absence_enforces_relation_and_target_only_field_disclosure() {
    let host = install_governed();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let access = |project: &str| ReadBoardAccess {
        project: project.to_owned(),
    };

    // 1. Authorized principal with relation (tag 1) + target-only `Gate` (tag 1) succeeds.
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
    let missing_rel = host
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
            missing_rel,
            WorthQueryApplicationRequestQueryDenial::Admission(ref cause)
                if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
        ),
        "unexpected denial: {missing_rel:?}"
    );

    // 3. Omitting `use_field_by(Gate)` (a field NEVER used on the root!) refuses
    //    at disclosure contract compilation with subject "Gate" on BOTH populated
    //    AND empty projects.
    for proj in ["p1", "empty"] {
        let missing_tfld = host
            .runtime()
            .request(&principal, &scope)
            .query(GovMissingTargetFieldIntent {
                project: proj.into(),
            })
            .execute_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                access(proj),
            )
            .err()
            .expect("missing target-only field disclosure rule must refuse");
        assert!(
            matches!(
                missing_tfld,
                WorthQueryApplicationRequestQueryDenial::Admission(ref cause)
                    if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
                        && cause.subject() == "Gate"
            ),
            "unexpected denial on {proj}: {missing_tfld:?}"
        );
    }

    // 4. Unauthorized relation disclosure (tag 2) refuses fail-closed with exact
    //    `InternalComputationDenied` on BOTH populated AND empty projects!
    for proj in ["p1", "empty"] {
        assert!(is_internal_computation_denied(
            host.runtime()
                .request(&principal, &scope)
                .query(GovDeniedRelIntent {
                    project: proj.into(),
                })
                .execute_governed(
                    ReadBoard::reference(),
                    InspectBoard::reference(),
                    access(proj),
                ),
        ));
    }

    // 5. Unauthorized target-only field `Gate` (root fields & BlockedBy are ALL
    //    tag 1; ONLY target-only `Gate` requires tag 2!) refuses fail-closed with
    //    exact `InternalComputationDenied` on BOTH populated AND empty projects!
    for proj in ["p1", "empty"] {
        assert!(is_internal_computation_denied(
            host.runtime()
                .request(&principal, &scope)
                .query(GovDeniedFieldIntent {
                    project: proj.into(),
                })
                .execute_governed(
                    ReadBoard::reference(),
                    InspectBoard::reference(),
                    access(proj),
                ),
        ));
    }
}
