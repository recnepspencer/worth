//! #119 PR A: an owned route's later round is a complete re-derivation of its
//! chain. Native folds it in place, so the commit equals the one-round answer,
//! or refuses with its typed #119 reason, publishing nothing.
use super::*;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_relational::facade::publication::{
    AuthoredRetirementRefusal as Cause, CoupledProvenancePostponement as Reason,
};

mod birth;
mod fold;
mod refusals;
mod settlement;

type Candidate<'a> = RelationalAuthoredCoupledCandidate<'a>;
type Failure = RelationalCoupledAssemblyFailure;
type RevisionStamps = Vec<(Option<u64>, Option<RelationalFieldRevision>)>;

struct Assemble<F>(F);
impl<F> RelationalAuthoredCoupledProducerAssembly for Assemble<F>
where
    F: Fn(&ExecutionRequest<'_, '_>, &mut Candidate<'_>) -> Result<(), Failure>,
{
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut Candidate<'_>,
    ) -> Result<ControlFlow<()>, Failure> {
        (self.0)(execution, candidate)?;
        Ok(ControlFlow::Continue(()))
    }
}
fn assemble<F>(rounds: F) -> Assemble<F>
where
    F: Fn(&ExecutionRequest<'_, '_>, &mut Candidate<'_>) -> Result<(), Failure>,
{
    Assemble(rounds)
}

fn named(key: &str, value: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)]),
    }))
}
fn creates(keys: &[&str], value: &str) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(named(key, &format!("{key}:{value}")))
        })]
}
fn fieldless_creates(keys: &[&str]) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(*key),
                fields: AspectFieldPatch::default(),
            })))
        })]
}
fn optional_struct_creates(keys: &[&str], value: &str) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(*key),
                fields: string_aspect_field_patch([(
                    aspect_key("summary"),
                    field_key("title"),
                    value,
                )]),
            })))
        })]
}
fn birth(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    keys: &[&str],
) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("owned", [3; 32]),
        absence: Some(world.absence(execution, "owned")),
        meaning: world.meaning("owned"),
        roles: keys.iter().map(|key| created(key)).collect(),
    })
}
fn birth_roles(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    roles: Vec<RequiredOutputRoleTarget>,
) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("owned", [3; 32]),
        absence: Some(world.absence(execution, "owned")),
        meaning: world.meaning("owned"),
        roles,
    })
}
fn preserve(
    world: &World,
    prior: &RelationalRequiredOutputPublication,
) -> RelationalCoupledProducerPosture {
    preserve_as(world, prior, RequiredOutputRolePosture::Preserve)
}
fn preserve_as(
    world: &World,
    prior: &RelationalRequiredOutputPublication,
    posture: RequiredOutputRolePosture,
) -> RelationalCoupledProducerPosture {
    let running = world.owner.admit_required_output("owned", [4; 32]);
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
        admission: world
            .owner
            .admit_required_output_succession(running, prior.authority())
            .unwrap(),
        predecessor: prior.clone(),
        meaning: preserve_meaning(prior),
        roles: vec![RequiredOutputRoleTarget {
            role: "output".into(),
            entity_name: "node".into(),
            kind: KindId(1),
            posture,
            target: EntityReference::Existing(output(prior)),
        }],
    })
}
fn preserve_meaning(
    prior: &RelationalRequiredOutputPublication,
) -> RequiredOutputPublicationMeaning {
    let mut meaning = prior.meaning().clone();
    meaning.source_epoch = worth_foundational::CanonicalDigestId::new([2; 32]);
    meaning
}
fn output(prior: &RelationalRequiredOutputPublication) -> EntityId {
    prior.members().next().unwrap().3
}
fn none() -> RelationalContentAttachments {
    RelationalContentAttachments::default()
}
/// One successor round of the route's chain, through its continuation token.
fn successor(
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    posture: RelationalCoupledProducerPosture,
    batches: Vec<WorkerIntentBatch>,
) -> Result<(), Failure> {
    let token = candidate.next_producer_round(execution, 0)?;
    candidate.append_successor_publication(execution, token, posture, &none(), batches)
}
fn refusal(err: TransactionCommitError) -> Cause {
    let TransactionCommitError::Publication { error, .. } = err else {
        panic!("a typed Native refusal: {err:?}")
    };
    error
        .authored_retirement_refusal()
        .expect("an authored refusal")
}
fn postponed(reason: Reason) -> Cause {
    Cause::CoupledContinuationProvenancePostponed { issue: 119, reason }
}
fn field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(aspect),
        CanonicalFieldPath::single(field_key(field)),
    )
}
fn revision_stamps(
    view: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    fields: &[(&str, &str)],
) -> RevisionStamps {
    fields
        .iter()
        .map(|(aspect, field)| {
            let aspect_ver = view
                .read_entity_aspect_version(execution, entity, &aspect_key(aspect))
                .unwrap()
                .unwrap();
            let field_rev = view
                .entity_field_revision_admitted(
                    execution,
                    entity,
                    &field_locator(aspect, field),
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
            (aspect_ver, field_rev)
        })
        .collect()
}
fn committed_revisions(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    fields: &[(&str, &str)],
) -> RevisionStamps {
    let view = world
        .runtime
        .read_truth()
        .project_observation(&world.basis().observation())
        .unwrap();
    revision_stamps(&view, execution, entity, fields)
}
/// Everything a commit publishes that the one-round answer fixes.
#[derive(Debug, PartialEq)]
struct Observed {
    /// Source, carrier and the route's outputs, read back.
    states: Vec<String>,
    changed: usize,
    /// Counted from the commit's canonical deltas.
    aspects: String,
    /// The published record patches.
    patches: String,
    /// The route's validated touches, from its recorded commands.
    touches: String,
}
/// Publish the one-chain proposal and observe it.
fn committed(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
    meaning: &RequiredOutputPublicationMeaning,
) -> (Observed, Vec<EntityId>) {
    assert_eq!(
        proposal.coupled_participants().count(),
        1,
        "one chain, one participant"
    );
    let touches = format!(
        "{:?}",
        proposal
            .coupled_producer_mutation_touches(execution, meaning)
            .expect("the route's touches")
            .unwrap()
            .touches()
    );
    let result = world.publish(execution, proposal);
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(origins.len(), 1, "one route publishes one origin");
    let mut ids = vec![world.source, world.carrier];
    ids.extend(origins[0].members().map(|member| member.3));
    let observed = Observed {
        states: states(&cold::facts(&world.runtime, execution, &ids)),
        changed: changed_entities(&result).len(),
        aspects: format!("{:?}", result.aspect_summary()),
        patches: format!(
            "{:?}",
            result
                .publication()
                .envelope
                .patch
                .authoritative_record_patches
        ),
        touches,
    };
    release_test_commit_snapshot(&world.runtime, &result);
    (observed, ids)
}
#[derive(Clone, Copy)]
enum Route<'k> {
    /// A Birth of these keys.
    Birth(&'k [&'k str]),
    /// A Preserve of a real earlier publication's output.
    Preserve,
}
/// The route's head round, then one successor round per later value, in
/// `world`. `round` authors a value's batches over the Preserve output.
fn chain(
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    route: Route<'_>,
    values: &[&str],
    round: &dyn Fn(&str, Option<EntityId>) -> Vec<WorkerIntentBatch>,
) -> (Observed, Vec<EntityId>) {
    chain_with_revisions(execution, world, route, values, round, &[]).0
}
/// Same as `chain`, also capturing candidate-pin and committed revision stamps
/// for the route's first output entity.
fn chain_with_revisions(
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    route: Route<'_>,
    values: &[&str],
    round: &dyn Fn(&str, Option<EntityId>) -> Vec<WorkerIntentBatch>,
    fields: &[(&str, &str)],
) -> ((Observed, Vec<EntityId>), RevisionStamps, RevisionStamps) {
    let prior =
        matches!(route, Route::Preserve).then(|| world.birth(execution, "owned", "owned-output"));
    let target = prior.as_ref().map(output);
    let meaning = prior
        .as_ref()
        .map_or_else(|| world.meaning("owned"), preserve_meaning);
    let birth_route = matches!(route, Route::Birth(_));
    let posture = |execution: &ExecutionRequest<'_, '_>| match route {
        Route::Birth(keys) => birth(world, execution, keys),
        Route::Preserve => preserve(world, prior.as_ref().unwrap()),
    };
    let pin_stamps = RefCell::new(Vec::new());
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    posture(execution),
                    &none(),
                    round(values[0], target),
                )?;
                let inventory = |candidate: &mut Candidate<'_>| {
                    candidate
                        .appended_output_entities(execution, "owned", &meaning)
                        .map(|outputs| outputs.expect("the appended route").to_vec())
                };
                let before = if birth_route {
                    inventory(candidate)?
                } else {
                    Vec::new()
                };
                for value in &values[1..] {
                    successor(
                        execution,
                        candidate,
                        posture(execution),
                        round(value, target),
                    )?;
                    if birth_route {
                        assert_eq!(inventory(candidate)?, before, "no second allocation");
                        assert!(candidate
                            .applied_output_input(execution, &meaning)?
                            .is_some());
                    }
                }
                if !fields.is_empty() {
                    let entity = target.unwrap_or_else(|| before[0]);
                    let view = candidate.pin_source(execution)?.project_candidate();
                    *pin_stamps.borrow_mut() = revision_stamps(&view, execution, entity, fields);
                }
                Ok(())
            }),
        )
        .unwrap();
    if birth_route {
        let contribution = proposal
            .coupled_contribution_records("owned", &meaning)
            .unwrap();
        assert!(contribution.publication_required());
    }
    let (observed, ids) = committed(world, execution, proposal, &meaning);
    let committed_stamps = if fields.is_empty() {
        Vec::new()
    } else {
        committed_revisions(world, execution, ids[2], fields)
    };
    ((observed, ids), pin_stamps.into_inner(), committed_stamps)
}
fn states(facts: &[worth_relational::facade::runtime::EntityReadRecord]) -> Vec<String> {
    facts
        .iter()
        .map(|fact| format!("{:?}", fact.authoritative_aspect_state))
        .collect()
}
