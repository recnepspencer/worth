use std::cell::Cell;
use std::convert::Infallible;

use worth_proof::contracts::*;

mod owner {
    worth_proof::authority_marker!(pub ContractAuthority);

    pub fn witness() -> worth_proof::AuthorityWitness<ContractAuthority> {
        ContractAuthority::witness()
    }
}

worth_proof::binding_axes! {
    pub struct RecoveryAxes {
        pub runtime: u64 => Runtime,
        pub branch: &'static str => Branch,
        pub principal: u32 => Principal,
    }
    drift pub enum RecoveryAxesDrift;
}

worth_proof::binding_axis_drift_certification! {
    binding: RecoveryAxes,
    drift: RecoveryAxesDrift,
    base: RecoveryAxes { runtime: 7, branch: "main", principal: 11 },
    twins: {
        runtime => Runtime = 8,
        branch => Branch = "other",
        principal => Principal = 12,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RecoveryTerminal {
    Resolved,
    Disposed,
}

impl TerminalState for RecoveryTerminal {
    fn label(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Disposed => "disposed",
        }
    }
}

struct Generations(Cell<u64>);

impl FreshnessSource for Generations {
    type Sample = u64;
    type Error = Infallible;

    fn sample(&self) -> Result<Self::Sample, Self::Error> {
        Ok(self.0.get())
    }
}

struct DeadlinePolicy;

impl FreshnessPolicy<Generations, u64> for DeadlinePolicy {
    fn classify(&self, deadline: &u64, observed_at: &u64) -> FreshnessVerdict {
        if observed_at < deadline {
            FreshnessVerdict::Current
        } else {
            FreshnessVerdict::AuthorityRevalidationRequired
        }
    }
}

struct Commit;
impl ActionMarker for Commit {}

struct Undo;
impl ActionMarker for Undo {}
impl InverseOf<Commit> for Undo {}

#[test]
fn brand_linear_freshness_and_causality_compose_on_the_public_lane() {
    let same_instance = with_brand(|brand| {
        let left = brand.bind(2_u8);
        let right = brand.bind(3_u8);
        left.into_value() + right.into_value()
    });
    assert_eq!(same_instance, 5);

    let resource = LinearResource::<_, RecoveryTerminal, owner::ContractAuthority>::mint(
        17_u64,
        &owner::witness(),
    );
    assert_eq!(
        resource.terminate(RecoveryTerminal::Resolved).label(),
        "resolved"
    );

    let source = Generations(Cell::new(4));
    let current = evaluate_freshness(5_u64, &source, &DeadlinePolicy)
        .expect("generation sampling is infallible");
    assert!(matches!(current, EvaluatedFreshness::Current(_)));
    source.0.set(5);
    let expired = evaluate_freshness(5_u64, &source, &DeadlinePolicy)
        .expect("generation sampling is infallible");
    assert!(matches!(
        expired,
        EvaluatedFreshness::AuthorityRevalidationRequired(_)
    ));

    let undo = Performed::<Undo, owner::ContractAuthority>::record(&owner::witness(), ());
    assert_eq!(redo(prove_inversion::<Undo, Commit, _, _>(&undo)), "redone");
    assert_eq!(publish(prove_derivation(&undo)), "published");
}

/// A redo lane reachable only by consuming proved inversion.
///
/// Takes the proof by value: the gate is the parameter, so there is no
/// `already_undone` flag a caller could set. Causality is not current
/// authority — an owner would still re-admit against live policy here.
fn redo(_proved: Inverts<Commit, owner::ContractAuthority>) -> &'static str {
    "redone"
}

/// A successor lane gated on the undo having actually run.
///
/// `DerivedFrom` borrows its predecessor, so one performed action may license
/// several successors without re-performing it.
fn publish(_derived: DerivedFrom<Undo, owner::ContractAuthority>) -> &'static str {
    "published"
}

#[test]
fn every_terminal_names_itself_and_a_resource_reaches_exactly_one() {
    let disposed = LinearResource::<_, RecoveryTerminal, owner::ContractAuthority>::mint(
        23_u64,
        &owner::witness(),
    )
    .terminate(RecoveryTerminal::Disposed);

    assert_eq!(disposed.id(), &23);
    assert_eq!(disposed.label(), "disposed");
    // The `Resolved` twin is exercised by the composition test above. A second
    // `terminate` on either resource does not typecheck; see
    // `tests/ui/contracts/compile_fail/linear_resource_cannot_terminate_twice.rs`.
}
