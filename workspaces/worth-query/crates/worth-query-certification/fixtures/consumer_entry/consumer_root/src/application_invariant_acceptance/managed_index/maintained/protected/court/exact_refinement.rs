//! Omitted inputs need influence permission even for non-Live exact membership.
use super::*;
#[test]
fn exact_refinement_governed_non_live_membership_checks_hidden_input_influence() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::exact_refinement_governed_non_live_membership_checks_hidden_input_influence"
        ),
    );
}
fn run() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls.clone())));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    create(
        &world,
        &principal,
        &scope,
        "exact-row",
        "shared",
        0x9199_2000,
    );
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let before = callbacks(&calls);
    let refused = request
        .query(query::exact::FindBlocked {
            directory: "catalog".into(),
            prefix: "sha".into(),
        })
        .execute_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .err()
        .expect("hidden Name lacks RowPresence influence");
    assert!(
        matches!(refused,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid)
    );
    assert_eq!(
        callbacks(&calls),
        before,
        "influence refusal precedes provider use"
    );
    for (probe, count) in [("sha", 0), ("shared", 1)] {
        let rows = request
            .query(query::exact::Find {
                directory: "catalog".into(),
                prefix: probe.into(),
            })
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                purpose(),
            )
            .unwrap();
        assert_eq!(rows.rows().len(), count);
        assert!(!rows.result_set_observation().unwrap().has_verified_realized_footprint(),
            "governed inputs retain conservative whole-Source consistency until passive purpose/capability binding is carried");
        if count == 1 {
            assert!(matches!(
                &rows.rows()[0].name,
                WorthQueryApplicationDisclosed::Omitted(_)
            ));
        }
    }
}
