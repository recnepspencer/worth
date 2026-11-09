//! Destruction keeps per-writer custody and the final current-ownership fence.
use super::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;
struct Write<'a> {
    world: &'a World,
    target: EntityId,
    destructive: bool,
    producer: &'static str,
}
impl RelationalAuthoredCoupledProducerAssembly for Write<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let batch = if self.destructive {
            WorkerIntentBatch::new("destructive-ordinary-write").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: self.target,
                }),
            ))
        } else {
            update(self.target, "legal-ordinary-field-update")
        };
        candidate.append_publication(
            execution,
            RelationalCoupledProducerPosture::Ordinary {
                admission: self
                    .world
                    .owner
                    .admit_required_output(self.producer, [4; 32]),
                absence: self.world.absence(execution, "ordinary"),
                meaning: self.world.meaning("ordinary"),
                roles: vec![existing(self.world.carrier)],
            },
            &RelationalContentAttachments::default(),
            vec![batch],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}
fn require_custody(
    error: TransactionCommitError,
    publication: &RelationalRequiredOutputPublication,
    target: EntityId,
) {
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("typed destructive custody refusal: {error:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::OwnedPublicationRequiresCascade {
            publication: worth_relational::facade::history::CommitId(
                publication.authority().publication_coordinate().0
            ),
            owner: publication.meaning().source_scope
        })
    );
    assert_eq!(
        error.context.affected_records,
        [
            RecordRef::Entity(target),
            RecordRef::Entity(publication.meaning().source_scope)
        ]
    );
}
#[test]
fn ordinary_destruction_refuses_at_the_part_writer_before_final_preparation() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let target = prior.members().next().unwrap().3;
    let before = world.basis();
    let error = world
        .prepare(
            execution,
            &Write {
                world: &world,
                target,
                destructive: true,
                producer: "ordinary",
            },
        )
        .unwrap_err();
    require_custody(error, &prior, target);
    assert_eq!(world.basis(), before);
}
#[test]
fn ordinary_destruction_reached_from_an_unrelated_source_refuses_at_the_pipeline_fence() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let target = prior.members().next().unwrap().3;
    world.source = world.carrier; // This source has no ownership closure of its own.
    let before = world.basis();
    let proposal = world
        .prepare(
            execution,
            &Write {
                world: &world,
                target,
                destructive: true,
                producer: "ordinary",
            },
        )
        .expect("the per-source roster does not claim this unrelated publication");
    let error = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap_err();
    require_custody(error, &prior, target);
    assert_eq!(world.basis(), before);
}
#[test]
fn an_ordinary_field_update_of_another_routes_owned_member_remains_legal() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let target = prior.members().next().unwrap().3;
    let proposal = world
        .prepare(
            execution,
            &Write {
                world: &world,
                target,
                destructive: false,
                producer: "ordinary",
            },
        )
        .unwrap();
    let result = world.publish(execution, proposal);
    assert_eq!(
        result
            .required_output_publications(&world.installed, execution)
            .count(),
        0,
        "a field write does not manufacture a successor origin"
    );
    release_test_commit_snapshot(&world.runtime, &result);
    assert!(world
        .runtime
        .readmit_current_owned_output_successor(
            execution,
            &world.installed,
            &world.basis(),
            "owned",
            prior.meaning()
        )
        .unwrap()
        .is_some());
}
#[test]
fn a_scope_owned_producer_cannot_hide_destructive_work_on_a_store_without_storage() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, false, true);
    let before = world.basis();
    // Native forbids creating an owned publication on this store in the first
    // place. Still attempt an actual destructive batch under an ordinary label;
    // the installed owner must refuse before it could conceal the missing store.
    let error = world
        .prepare(
            execution,
            &Write {
                world: &world,
                target: world.carrier,
                destructive: true,
                producer: "owned",
            },
        )
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::CurrentOwnershipStorageMissing)
    );
    assert_eq!(world.basis(), before);
}
