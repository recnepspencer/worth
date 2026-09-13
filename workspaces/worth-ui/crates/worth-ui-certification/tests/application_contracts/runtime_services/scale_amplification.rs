use std::collections::HashMap;

/// An occupancy index modelled independently of the production one.
///
/// `RS-10` requires an independent model oracle, so this structure shares no
/// code with `UiServiceProposalOccupancyTable`. It charges work by one explicit
/// rule: examining a neighborhood other than the one an operation is keyed to
/// costs a foreign examination. A keyed planner therefore reports zero and a
/// sweeping planner cannot, which is what makes the comparison falsifiable.
#[derive(Default)]
struct IndependentOccupancyModel {
    neighborhoods: HashMap<u64, Vec<(usize, u64)>>,
    requirements_visited: u64,
    foreign_neighborhoods_examined: u64,
}

impl IndependentOccupancyModel {
    /// Charges one neighborhood examination performed by `owner`'s operation.
    fn examine(&mut self, owner: u64, examined: u64) {
        if owner != examined {
            self.foreign_neighborhoods_examined += 1;
        }
    }

    /// One proposal reserving `families` entries at `scope`, keyed straight into
    /// its own neighborhood.
    fn reserve_keyed(&mut self, neighborhood: u64, scope: u64, families: usize) {
        self.examine(neighborhood, neighborhood);
        for family in 0..families {
            self.requirements_visited += 1;
            let entries = self.neighborhoods.entry(neighborhood).or_default();
            assert!(
                !entries.contains(&(family, scope)),
                "an independent neighborhood cannot double-book one family scope"
            );
            entries.push((family, scope));
        }
    }

    /// The same reservation performed by a planner that scans the whole index
    /// before keying. Used only to prove the model can distinguish the two.
    fn reserve_by_sweep(&mut self, neighborhood: u64, scope: u64, families: usize) {
        for examined in self.neighborhoods.keys().copied().collect::<Vec<_>>() {
            self.examine(neighborhood, examined);
        }
        self.reserve_keyed(neighborhood, scope, families);
    }

    fn neighborhoods(&self) -> u64 {
        self.neighborhoods.len() as u64
    }
}

/// The declared `RS-10` proposal world: 64 sibling neighborhoods of one
/// application, each compiling all six service families at one scope.
const SCALE_NEIGHBORHOODS: u64 = 64;
const SCALE_FAMILIES: usize = 6;

fn independent_proposal_oracle() -> IndependentOccupancyModel {
    let mut model = IndependentOccupancyModel::default();
    for neighborhood in 0..SCALE_NEIGHBORHOODS {
        model.reserve_keyed(neighborhood, neighborhood + 1, SCALE_FAMILIES);
    }
    model
}

#[test]
#[ignore = "closure-stress: milestone 3.15 RS-10 full service and mounted scale world"]
fn runtime_service_scale_has_named_local_work_and_exact_zero_residue() {
    let evidence = worth_ui_test_support::runtime_service_scale_evidence();
    let model = independent_proposal_oracle();

    // Scale relationships, each read back from live owner state.
    assert_eq!(evidence.service_neighborhoods(), 64);
    assert_eq!(evidence.commands(), 4_096);
    assert_eq!(evidence.focus_participants(), 128);
    assert_eq!(evidence.selection_keys(), 1_024);
    assert_eq!(evidence.scroll_owners(), 8);
    assert_eq!(evidence.portal_layers(), 4);
    assert_eq!(evidence.active_motion_tracks(), 64);

    // Bounded local work.
    assert_eq!(evidence.portal_neighborhoods_visited(), 4);
    assert_eq!(evidence.focus_participants_visited(), 1);
    assert_eq!(evidence.motion_tracks_sampled(), 64);
    assert_eq!(evidence.scroll_chain_depth_visited(), 8);
    assert_eq!(evidence.selection_keys_visited(), 1);
    assert_eq!(evidence.command_candidates_resolved(), 1);

    // Inactive retention costs no per-frame work, measured on the same sampler
    // that still retains all sixty-four completed tracks.
    assert_eq!(evidence.retained_inactive_motion_tracks(), 64);
    assert_eq!(evidence.completed_motion_terminals(), 64);
    assert_eq!(evidence.inactive_motion_tracks_sampled(), 0);

    // The proposal compiler's bounded work is compared against the independent
    // model rather than restated as a literal.
    assert_eq!(evidence.service_neighborhoods(), model.neighborhoods());
    assert_eq!(
        evidence.proposal_requirements_visited(),
        model.requirements_visited,
        "production requirement visits must match an independently keyed index"
    );
    assert_eq!(
        evidence.unrelated_neighborhoods_touched(),
        model.foreign_neighborhoods_examined,
        "an ordinary reserve keys into its own neighborhood and sweeps no sibling"
    );

    assert!(evidence.terminal_resources_zero());
}

#[test]
fn the_independent_occupancy_model_detects_a_sweeping_planner() {
    // The model is a useful oracle only if a sweeping implementation would
    // disagree with it, so the same world is replayed both ways.
    let keyed = independent_proposal_oracle();
    assert_eq!(keyed.foreign_neighborhoods_examined, 0);
    assert_eq!(
        keyed.requirements_visited,
        SCALE_NEIGHBORHOODS * SCALE_FAMILIES as u64
    );

    let mut swept = IndependentOccupancyModel::default();
    for neighborhood in 0..SCALE_NEIGHBORHOODS {
        swept.reserve_by_sweep(neighborhood, neighborhood + 1, SCALE_FAMILIES);
    }

    assert_eq!(
        swept.requirements_visited, keyed.requirements_visited,
        "a sweep performs the same requirement work and differs only in breadth"
    );
    assert_eq!(
        swept.foreign_neighborhoods_examined,
        SCALE_NEIGHBORHOODS * (SCALE_NEIGHBORHOODS - 1) / 2,
        "a sweeping planner charges every sibling it examined"
    );
    assert!(swept.foreign_neighborhoods_examined > keyed.foreign_neighborhoods_examined);
}
