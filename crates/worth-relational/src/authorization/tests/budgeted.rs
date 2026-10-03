use super::{allow_plan, authorization_fixture};
use crate::authorization::{
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationFreshness,
};
use crate::facade::history::BranchId;
use crate::tests::support::delete_relation_on_branch;

#[derive(Debug, Eq, PartialEq)]
enum Exhausted {
    Work,
    Preparation,
}

struct Budget {
    work: u64,
    bytes: u64,
    work_limit: u64,
    byte_limit: u64,
}
impl Budget {
    fn unlimited() -> Self {
        Self {
            work: 0,
            bytes: 0,
            work_limit: u64::MAX,
            byte_limit: u64::MAX,
        }
    }
}
impl RelationalAuthorizationObservationAdmission for Budget {
    type Stop = Exhausted;
    fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Exhausted> {
        let next_work = self.work.checked_add(work).ok_or(Exhausted::Work)?;
        let next_bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(Exhausted::Preparation)?;
        if next_work > self.work_limit {
            return Err(Exhausted::Work);
        }
        if next_bytes > self.byte_limit {
            return Err(Exhausted::Preparation);
        }
        self.work = next_work;
        self.bytes = next_bytes;
        Ok(())
    }
}

#[test]
fn budgeted_observer_matches_ordinary_authorization_and_held_old_root() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let ordinary = fixture
        .runtime
        .observe_authorization(allow_plan(
            snapshot.clone(),
            fixture.principal,
            fixture.scope,
            [],
        ))
        .unwrap();
    let observe = |snapshot, budget: &mut Budget| {
        fixture.runtime.observe_authorization_budgeted(
            allow_plan(snapshot, fixture.principal, fixture.scope, []),
            budget,
        )
    };
    let bounded = observe(snapshot.clone(), &mut Budget::unlimited()).unwrap();
    assert_eq!(bounded.paths(), ordinary.paths());
    assert_eq!(bounded.counters(), ordinary.counters());
    assert!(bounded.paths()[0].matched());
    let revoked = delete_relation_on_branch(
        &fixture.runtime,
        fixture.role_scope_relation,
        BranchId("main".to_string()),
    );
    let old = observe(snapshot, &mut Budget::unlimited()).unwrap();
    assert_eq!(old.paths(), ordinary.paths());
    assert_eq!(old.counters(), ordinary.counters());
    let current = observe(revoked.snapshot.clone(), &mut Budget::unlimited()).unwrap();
    assert!(!current.paths()[0].matched());
}

#[test]
fn budgeted_currentness_preserves_legacy_result_and_carried_refusal() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let evidence = fixture
        .runtime
        .observe_authorization(allow_plan(
            snapshot.clone(),
            fixture.principal,
            fixture.scope,
            [],
        ))
        .unwrap();
    let mut measured = Budget::unlimited();
    assert_eq!(
        fixture.runtime.compare_authorization_observation_budgeted(
            &evidence,
            &snapshot,
            &mut measured
        ),
        Ok(RelationalAuthorizationObservationFreshness::Fresh)
    );
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&evidence, snapshot.clone()),
        RelationalAuthorizationObservationFreshness::Fresh
    );
    assert!(measured.work > 1 && measured.bytes > 0);
    let mut late_work = Budget {
        work_limit: measured.work - 1,
        ..Budget::unlimited()
    };
    assert!(matches!(
        fixture.runtime.compare_authorization_observation_budgeted(
            &evidence,
            &snapshot,
            &mut late_work,
        ),
        Err(Stop::Admission(Exhausted::Work))
    ));
    assert!(late_work.work > 0);
    let mut late_bytes = Budget {
        byte_limit: measured.bytes - 1,
        ..Budget::unlimited()
    };
    assert!(matches!(
        fixture.runtime.compare_authorization_observation_budgeted(
            &evidence,
            &snapshot,
            &mut late_bytes,
        ),
        Err(Stop::Admission(Exhausted::Preparation))
    ));
    let revoked = delete_relation_on_branch(
        &fixture.runtime,
        fixture.role_scope_relation,
        BranchId("main".to_string()),
    );
    assert_eq!(
        fixture.runtime.compare_authorization_observation_budgeted(
            &evidence,
            &revoked.snapshot,
            &mut Budget::unlimited(),
        ),
        Ok(RelationalAuthorizationObservationFreshness::Stale)
    );
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&evidence, revoked.snapshot.clone()),
        RelationalAuthorizationObservationFreshness::Stale
    );
}

#[test]
fn carried_stop_survives_initial_and_late_work_or_preparation_exhaustion() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let observe = |budget: &mut Budget| {
        fixture.runtime.observe_authorization_budgeted(
            allow_plan(snapshot.clone(), fixture.principal, fixture.scope, []),
            budget,
        )
    };
    let mut measured = Budget::unlimited();
    assert!(observe(&mut measured).unwrap().paths()[0].matched());
    assert!(measured.work > 1 && measured.bytes > 0);
    let mut initial = Budget {
        work_limit: 0,
        ..Budget::unlimited()
    };
    assert!(matches!(
        observe(&mut initial),
        Err(Stop::Admission(Exhausted::Work))
    ));
    assert_eq!(initial.work, 0);
    let mut late = Budget {
        work_limit: measured.work - 1,
        ..Budget::unlimited()
    };
    assert!(matches!(
        observe(&mut late),
        Err(Stop::Admission(Exhausted::Work))
    ));
    assert!(late.work > 1);
    let mut bytes = Budget {
        byte_limit: measured.bytes - 1,
        ..Budget::unlimited()
    };
    assert!(matches!(
        observe(&mut bytes),
        Err(Stop::Admission(Exhausted::Preparation))
    ));
    assert!(bytes.bytes < measured.bytes);
    assert_eq!(
        fixture.runtime.visibility_authority().snapshot().version_id,
        snapshot.version_id
    );
}

#[test]
fn reconstructed_projection_refuses_exact_borrow_without_policy_denial() {
    use crate::runtime::RelationalBorrowedRecordReadDenial;
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let reconstructed = fixture
        .runtime
        .read_truth()
        .try_project_historical_version(snapshot.version_id)
        .unwrap();
    let mut called = false;
    let result = reconstructed.with_exact_entity_state(fixture.principal, |_, _| {
        called = true;
    });
    assert_eq!(
        result,
        Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired)
    );
    assert!(!called);
    assert_eq!(
        reconstructed.exact_relation_metadata(fixture.role_scope_relation),
        Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired)
    );
}
