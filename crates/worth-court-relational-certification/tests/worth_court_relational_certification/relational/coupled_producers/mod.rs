//! Ordinary producer participation has no lifetime publication authority.
use std::{cell::RefCell, ops::ControlFlow};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, content::RelationalContentAttachments, identity::*,
    mvcc::*, runtime::*, symbols::ClientKey, transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
        string_aspect_field_patch,
    },
};
mod fixture;
use fixture::*;

struct Ordinary<'a> {
    world: &'a World,
    family: &'static str,
    absence: RefCell<Option<RelationalRequiredOutputRouteAbsence>>,
    existing: bool,
    duplicate: bool,
    mixed: bool,
}
impl RelationalAuthoredCoupledProducerAssembly for Ordinary<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let absence = self
            .absence
            .borrow_mut()
            .take()
            .unwrap_or_else(|| self.world.absence(execution, self.family));
        let mut roles = vec![created("ordinary-companion")];
        if self.existing {
            roles.push(existing(self.world.carrier));
        }
        candidate.append_publication(
            execution,
            RelationalCoupledProducerPosture::Ordinary {
                admission: self.world.owner.admit_required_output(self.family, [1; 32]),
                absence,
                meaning: self.world.meaning(self.family),
                roles,
            },
            &RelationalContentAttachments::default(),
            vec![update(self.world.carrier, "calculated-carrier")
                .push(create_intent("ordinary-companion"))],
        )?;
        if self.duplicate {
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Ordinary {
                    admission: self.world.owner.admit_required_output(self.family, [2; 32]),
                    absence: self.world.absence(execution, self.family),
                    meaning: self.world.meaning(self.family),
                    roles: vec![existing(self.world.carrier)],
                },
                &RelationalContentAttachments::default(),
                vec![WorkerIntentBatch::new("duplicate")],
            )?;
        }
        if self.mixed {
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                    admission: self.world.owner.admit_required_output("owned", [3; 32]),
                    absence: Some(self.world.absence(execution, "owned")),
                    meaning: self.world.meaning("owned"),
                    roles: vec![created("owned-output")],
                }),
                &RelationalContentAttachments::default(),
                vec![create("owned-output")],
            )?;
        }
        Ok(ControlFlow::Continue(()))
    }
}
fn request() -> worth_execution::SerialRequest {
    worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(
    )
}
fn assembly(world: &World) -> Ordinary<'_> {
    Ordinary {
        world,
        family: "ordinary",
        absence: RefCell::new(None),
        existing: true,
        duplicate: false,
        mixed: false,
    }
}
#[test]
fn ordinary_carrier_and_companions_acquire_no_publication_ownership() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    for storage in [false, true] {
        let world = World::new(execution, storage, false);
        let proposal = world.prepare(execution, &assembly(&world)).unwrap();
        let participants = proposal.coupled_participants().collect::<Vec<_>>();
        assert_eq!(
            participants.len(),
            1,
            "ordinary writers remain settlement participants"
        );
        let contribution = proposal
            .coupled_contribution_records("ordinary", &world.meaning("ordinary"))
            .unwrap();
        assert!(!contribution.publication_required());
        assert!(!contribution.records().is_empty());
        let result = world.publish(execution, proposal);
        assert_eq!(
            result
                .required_output_publications(&world.installed, execution)
                .count(),
            0
        );
        assert!(world
            .runtime
            .readmit_required_output_route_absence(
                execution,
                &world.installed,
                &world.basis(),
                &world.meaning("ordinary")
            )
            .unwrap()
            .is_some());
        release_test_commit_snapshot(&world.runtime, &result);
    }
}
#[test]
fn ordinary_and_owned_parts_publish_only_the_owned_origin() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let mut parts = assembly(&world);
    parts.mixed = true;
    let proposal = world.prepare(execution, &parts).unwrap();
    assert_eq!(proposal.coupled_participants().count(), 2);
    let result = world.publish(execution, proposal);
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].producer(), "owned");
    assert!(world
        .runtime
        .readmit_required_output_route_absence(
            execution,
            &world.installed,
            &world.basis(),
            &world.meaning("ordinary")
        )
        .unwrap()
        .is_some());
    release_test_commit_snapshot(&world.runtime, &result);
}
#[test]
fn ordinary_label_refuses_all_created_scope_owned_and_repeated_routes_without_effect() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    for kind in 0..3 {
        let before = world.basis();
        let mut parts = assembly(&world);
        if kind == 0 {
            parts.existing = false;
        } else if kind == 1 {
            parts.family = "owned";
        } else {
            parts.duplicate = true;
        }
        let err = world.prepare(execution, &parts).unwrap_err();
        let TransactionCommitError::Publication { error, .. } = err else {
            panic!("typed Native admission refusal: {err:?}")
        };
        use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(if kind == 2 {
                Cause::RepeatedCoupledOwner
            } else {
                Cause::ForeignCoupledOwner
            })
        );
        assert_eq!(
            world.basis(),
            before,
            "no partial source or producer effect at the immutable C root"
        );
    }
}

mod admission;
mod destruction;

mod cold;

mod mixed;

mod stages;

mod cold_candidate_pin;
