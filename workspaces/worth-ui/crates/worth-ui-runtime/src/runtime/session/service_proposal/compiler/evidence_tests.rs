use std::collections::BTreeMap;

#[test]
fn same_session_generations_occupy_exact_independent_neighborhoods() {
    let mut compiler = super::UiServiceProposalCompiler::new();
    let first = super::super::fixture_service_request_coherence(130);
    let axes = first.axes();
    let second = worth_proof::Binding::new(super::super::UiServiceRequestCoherenceAxes {
        application: super::super::fixture_application_generation_in_session(130, 131),
        semantic_surface: axes.semantic_surface,
        host_surface: axes.host_surface,
        binding: axes.binding,
        presentation: axes.presentation,
        origin: axes.origin,
        causal_parent: axes.causal_parent,
        causal_root: axes.causal_root,
        source_order: axes.source_order,
        cancellation: axes.cancellation,
        resource_budget: axes.resource_budget,
    });
    let support = all_service_support();
    let first = compiler
        .preflight(six_family_candidate(1_130, 1, &first), &first, support)
        .unwrap();
    let first = match compiler.reserve(first).unwrap() {
        super::UiServiceProposalReservationOutcome::Reserved(reservation) => reservation,
        super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
    };
    let second = compiler
        .preflight(six_family_candidate(1_131, 1, &second), &second, support)
        .unwrap();
    let second = match compiler.reserve(second).unwrap() {
        super::UiServiceProposalReservationOutcome::Reserved(reservation) => reservation,
        super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
    };

    assert_eq!(compiler.live_occupancy_count(), 12);
    compiler.shutdown_reservation(first).unwrap();
    assert_eq!(compiler.live_occupancy_count(), 6);
    compiler.shutdown_reservation(second).unwrap();
    assert!(compiler.census().is_zero());
}

#[test]
fn sixty_four_independent_neighborhoods_have_linear_local_work_and_zero_residue() {
    let mut compiler = super::UiServiceProposalCompiler::new();
    let base_coherence = super::super::fixture_service_request_coherence(120);
    let support = all_service_support();
    let mut independent_model = BTreeMap::new();
    let mut reservations = Vec::new();

    for index in 0_u64..64 {
        let scope = index + 1;
        let coherence = coherence_in_fresh_surface(&base_coherence);
        assert_eq!(independent_model.insert(scope, 1_000 + index), None);
        let candidate = six_family_candidate(1_000 + index, scope, &coherence);
        let preflighted = compiler.preflight(candidate, &coherence, support).unwrap();
        let reservation = match compiler.reserve(preflighted).unwrap() {
            super::UiServiceProposalReservationOutcome::Reserved(reservation) => reservation,
            super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
        };
        reservations.push(reservation);
        let expected = u16::try_from(reservations.len()).unwrap();
        assert_eq!(
            compiler.census().entries(),
            [
                ("proposals", expected),
                ("occupancy_leases", expected * 6),
                ("cancellation_records", expected),
                ("stage_receipts", 0),
            ]
        );
    }

    let counters = compiler.occupancy_work_counters();
    assert_eq!(counters.proposal_requirements_visited(), 384);
    assert_eq!(counters.unrelated_neighborhoods_touched(), 0);
    assert_eq!(compiler.live_neighborhood_count(), 64);
    assert_eq!(independent_model.len(), 64);

    let overflow_coherence = coherence_in_fresh_surface(&base_coherence);
    let overflow = compiler
        .preflight(
            six_family_candidate(9_999, 65, &overflow_coherence),
            &overflow_coherence,
            support,
        )
        .unwrap();
    let before_overflow = compiler.census();
    assert!(matches!(
        compiler.reserve(overflow),
        Err(super::UiServiceProposalReservationDenial::Occupancy(
            super::super::UiServiceProposalOccupancyDenial::CapacityExceeded
        ))
    ));
    assert_eq!(compiler.census(), before_overflow);
    assert_eq!(
        compiler
            .occupancy_work_counters()
            .proposal_requirements_visited(),
        390
    );
    while let Some(reservation) = reservations.pop() {
        compiler.shutdown_reservation(reservation).unwrap();
    }
    assert!(compiler.census().is_zero());
    assert_eq!(compiler.live_occupancy_count(), 0);
    assert_eq!(compiler.live_cancellation_count(), 0);
}

/// The foreign-neighborhood counter is not structurally pinned to zero: a path
/// that sweeps the index instead of keying into its own neighborhood charges
/// every other neighborhood it examined. Without this probe, the `RS-10`
/// assertion that an ordinary reserve touches zero would be unfalsifiable.
#[test]
fn foreign_neighborhood_work_is_observable_and_not_pinned_to_zero() {
    let mut compiler = super::UiServiceProposalCompiler::new();
    let support = all_service_support();
    let base = super::super::fixture_service_request_coherence(700);
    let mut reservations = Vec::new();
    for index in 0..3_u64 {
        let coherence = coherence_in_fresh_surface(&base);
        let preflighted = compiler
            .preflight(
                six_family_candidate(7_000 + index, index + 1, &coherence),
                &coherence,
                support,
            )
            .unwrap();
        match compiler.reserve(preflighted).unwrap() {
            super::UiServiceProposalReservationOutcome::Reserved(reservation) => {
                reservations.push(reservation);
            }
            super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
        }
    }

    assert_eq!(compiler.live_neighborhood_count(), 3);
    assert_eq!(
        compiler
            .occupancy_work_counters()
            .unrelated_neighborhoods_touched(),
        0,
        "keyed reserves examine only their own neighborhood"
    );

    let before = compiler
        .occupancy_work_counters()
        .unrelated_neighborhoods_touched();
    compiler
        .shutdown_all_before_effect()
        .expect("before-effect shutdown sweeps and releases every neighborhood");
    let after = compiler
        .occupancy_work_counters()
        .unrelated_neighborhoods_touched();

    assert!(
        after > before,
        "a full index sweep must move the foreign-neighborhood counter"
    );
    for reservation in reservations {
        core::mem::drop(reservation);
    }
}

fn coherence_in_fresh_surface(
    base: &super::super::UiServiceRequestCoherence,
) -> super::super::UiServiceRequestCoherence {
    let axes = base.axes();
    worth_proof::Binding::new(super::super::UiServiceRequestCoherenceAxes {
        application: axes.application.clone(),
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .unwrap(),
        host_surface: axes.host_surface,
        binding: axes.binding,
        presentation: axes.presentation,
        origin: axes.origin,
        causal_parent: axes.causal_parent,
        causal_root: axes.causal_root,
        source_order: axes.source_order,
        cancellation: axes.cancellation,
        resource_budget: axes.resource_budget,
    })
}

fn six_family_candidate(
    identity: u64,
    scope: u64,
    coherence: &super::super::UiServiceRequestCoherence,
) -> super::UiServiceProposalCandidate {
    let families = crate::capability::UiRuntimeServiceFamily::ALL
        .into_iter()
        .map(|family| super::UiServiceFamilyProposal::recorded_fixture(family, scope, 1, 1, 1))
        .collect();
    super::UiServiceProposalCandidate::for_test(
        identity,
        super::UiServiceProposalDemand::recorded_fixture(
            super::super::fixture_service_family_participation(6),
            6,
            6,
            6,
        ),
        coherence.clone(),
        families,
    )
}

fn all_service_support() -> crate::capability::UiRuntimeServiceSupport {
    crate::capability::UiRuntimeServiceFamily::ALL
        .into_iter()
        .fold(
            crate::capability::UiRuntimeServiceSupport::none_installed(),
            crate::capability::UiRuntimeServiceSupport::with_installed,
        )
}

#[test]
fn compiler_source_contains_no_publication_or_host_effect_authority_lane() {
    let sources = [
        include_str!("mod.rs"),
        include_str!("settlement_compiler.rs"),
        include_str!("staging.rs"),
        include_str!("settlement.rs"),
        include_str!("receipt.rs"),
        include_str!("preflight.rs"),
        include_str!("proposal.rs"),
        include_str!("family_proposal.rs"),
        include_str!("reservation.rs"),
        include_str!("staged_reference.rs"),
        include_str!("dependency.rs"),
        include_str!("../occupancy.rs"),
        include_str!("../occupancy/lease.rs"),
        include_str!("../occupancy/neighborhood.rs"),
        include_str!("../cancellation.rs"),
        include_str!("../census.rs"),
    ];
    for source in sources {
        let normalized = source.to_ascii_lowercase();
        for forbidden in [
            "worth_query",
            "worth-query",
            "replay",
            "uimountedframe",
            "hosteffect",
            "host_effect",
            ".publish(",
            "settle_host",
            "authoritymarker",
            "box<dyn any",
        ] {
            assert!(
                !normalized.contains(forbidden),
                "compiler source gained forbidden authority/payload token {forbidden}"
            );
        }
        for family in [
            "portal",
            "scroll",
            "focus",
            "selection",
            "commandrouting",
            "motion",
        ] {
            assert!(
                !normalized.contains(&format!("uiruntimeservicefamily::{family} =>")),
                "compiler must not switch on {family} behavior"
            );
        }
    }
}

#[test]
fn session_compiler_is_inert_when_unused() {
    let compiler = super::UiServiceProposalCompiler::new();
    assert!(compiler.census().is_zero());
    assert_eq!(compiler.live_occupancy_count(), 0);
    assert_eq!(compiler.live_cancellation_count(), 0);
}
