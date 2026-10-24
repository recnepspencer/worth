//! Native current-root suspension restores extra only from fresh corrected authored rows.
use super::*;
use binding::{PreservedExtra, PreservedSupport, PreservedTotal};
use members::{SupportKey, SupportValue, TotalSupport};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
fn evicted_extra_is_reconstructed_from_fresh_corrected_authored_source() {
    let _guard = checkpoint_recovery_test_guard();
    let mut app = correction::install(0);
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    let original = correction::settled(&app, &request, true);
    let source = correction::source(&app, &request);
    correction::settled(&app, &request, false);
    let inverse = correction::reverse(&app, &request, &source);
    let corrected = correction::settled(&app, &request, false);
    correction::assert_restored(&original, &corrected);
    let observation = request.retain_current_read().unwrap();
    let authored = request
        .at(&observation)
        .query(source_query::SourceRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        authored.receipt().inspect().read_work().truncation_count(),
        0
    );
    let rows = authored.rows()[0].clone();
    let authored_version = authored.receipt().inspect().basis().version();
    drop(authored);
    let input = request
        .at(&observation)
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        input.receipt().inspect().basis().version(),
        authored_version,
        "fresh authored rows and producer source use the same native component basis"
    );
    let source_observation = input.observed_sources()[0].clone();
    drop(input);
    let selected = crate::checkpoint_recovery::support::host_call(&app, |phase| {
        app.on_branch(app.current_world()).select(phase)
    })
    .unwrap();
    assert_eq!(
        selected.product().selected_commit(),
        observation.selected_commit()
    );
    let suspended = selected
        .suspend_current_generated_output::<producer::PreserveProducer>(&scope, source_observation)
        .unwrap_or_else(|_| panic!("the recreated optional member owns real generated payload"));
    let refusal = request
        .query(family_query::FamilyRead {
            body_key: output_key(SCOPE),
        })
        .execute()
        .err()
        .expect("ordinary current family read reaches unavailable current-root materialization");
    assert!(
        matches!(refusal,
            WorthQueryApplicationRequestQueryDenial::PrincipalResolution(ref denial)
                if denial.kind() == WorthQueryPrincipalResolutionDenialKind::Refused(WorthQueryPrincipalResolutionRefusalKind::BranchMaterializationSuspended)
        ) || matches!(refusal,
            WorthQueryApplicationRequestQueryDenial::Admission(ref denial)
                if denial.kind() == WorthQueryApplicationQueryAdmissionDenialKind::BranchMaterializationSuspended
        ),
        "the native materialization refusal is preserved: {refusal:?}"
    );
    let value = rows.total("even");
    assert_eq!(value, 2 + 3 + 8, "independent authored arithmetic");
    let mut reconstruction = app
        .with_host_advancement(|phase| {
            app.reconstruct_generated_output::<producer::PreserveProducer>(&phase, suspended)
                .unwrap_or_else(|_| panic!("the original manifest admits reconstruction"))
        })
        .unwrap();
    let WorthQueryReconstructedOutputEntity::Generated(extra) = reconstruction
        .output_member::<PreservedExtra<CheckpointSchema>>("extra")
        .unwrap()
    else {
        panic!("only extra is generated");
    };
    let WorthQueryReconstructedOutputEntity::Retained(total) = reconstruction
        .output::<PreservedTotal<CheckpointSchema>>()
        .unwrap()
    else {
        panic!("the ring total remains retained");
    };
    for suffix in ["b", "c"] {
        assert!(matches!(
            reconstruction
                .output_member::<PreservedSupport<CheckpointSchema>>(suffix)
                .unwrap(),
            WorthQueryReconstructedOutputEntity::Retained(_)
        ));
    }
    reconstruction
        .field(
            &extra,
            SupportKey::reference(),
            members::key(&output_key(SCOPE)),
        )
        .unwrap();
    reconstruction
        .field(&extra, SupportValue::reference(), value)
        .unwrap();
    reconstruction
        .relation_from_retained(TotalSupport::reference(), &total, &extra)
        .unwrap();
    let completed = reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("every suspended record is claimed once"));
    app.restore_generated_output(completed, &scope)
        .unwrap_or_else(|_| panic!("the original suspension basis publishes through native CAS"));
    let after = correction::settled(&app, &request, false);
    assert_eq!(after, corrected, "ordinary owning demand explicitly delivers restored extra with exact native correspondence");
    drop((rows, observation, inverse, source, principal, scope));
    app.close_conditional_runtime().unwrap();
}
