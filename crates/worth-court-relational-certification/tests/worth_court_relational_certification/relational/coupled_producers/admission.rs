//! Current Native presence cannot be demoted by an ordinary label.
use super::*;
#[test]
fn published_route_rejects_an_old_absence_in_an_ordinary_part() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, false);
    let absence = world.absence(execution, "ordinary");
    let prior = world.birth(execution, "ordinary", "first-owned-output");
    let before = world.basis();
    let part = assembly(&world);
    *part.absence.borrow_mut() = Some(absence);
    let error = world.prepare(execution, &part).unwrap_err();
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("typed current route refusal: {error:?}")
    };
    assert!(
        error.detail.contains("requires its exact predecessor"),
        "{error:?}"
    );
    assert_eq!(world.basis(), before);
    assert!(world
        .runtime
        .readmit_required_output_route_absence(
            execution,
            &world.installed,
            &before,
            prior.meaning()
        )
        .unwrap()
        .is_none());
}

struct RepeatedBirth<'a>(&'a World);
impl RelationalAuthoredCoupledProducerAssembly for RepeatedBirth<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        if assembly(self.0).assemble(execution, candidate)?.is_break() {
            return Ok(ControlFlow::Break(()));
        }
        candidate.append_publication(
            execution,
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                admission: self.0.owner.admit_required_output("ordinary", [2; 32]),
                absence: Some(self.0.absence(execution, "ordinary")),
                meaning: self.0.meaning("ordinary"),
                roles: vec![created("second-writer")],
            }),
            &RelationalContentAttachments::default(),
            vec![create("second-writer")],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn ordinary_and_birth_writers_of_the_same_route_refuse_without_effect() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, false);
    let before = world.basis();
    let error = world
        .prepare(execution, &RepeatedBirth(&world))
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("typed duplicate route refusal: {error:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(
            worth_relational::facade::publication::AuthoredRetirementRefusal::RepeatedCoupledOwner
        )
    );
    assert_eq!(world.basis(), before);
}
