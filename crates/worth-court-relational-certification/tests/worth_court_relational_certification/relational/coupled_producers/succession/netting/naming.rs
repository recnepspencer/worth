//! C-10: a successor round's naming plan replaces its superseded round's
//! (#119 §2.6 Naming): only the live round's births execute.
use super::super::naming::{named_registry, named_world, seeds};
use super::*;
use worth_relational::facade::lineage::{
    CommitNamingPlan, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
    PersistentNameOfRecord,
};

const NODE_SCHEME: &str = "worth.cert.coupled.node.v1";

fn births(keys: &[&str], occurrence: &str) -> CommitNamingPlan {
    CommitNamingPlan {
        pending_births: keys
            .iter()
            .map(|key| PendingCommitBirth {
                target: PendingBirthTarget::CreatedEntity(target(key)),
                scheme_id: NODE_SCHEME.to_owned(),
                occurrence_key: format!("{occurrence}.{key}"),
                role_path: "authored".to_owned(),
                parent_identities: Vec::new(),
                referenced_names: Vec::new(),
                canonical_discriminator: Vec::new(),
            })
            .collect(),
        ..CommitNamingPlan::default()
    }
}

/// The named rounds' commit: each output's role, identity and live name.
fn named(rounds: &[(&[&str], &str)]) -> Vec<(String, EntityId, String)> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = named_world(execution, named_registry(), seeds()).0;
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                for &(keys, occurrence) in rounds {
                    let posture = birth(&world, execution, keys);
                    let plan = Some(births(keys, occurrence));
                    if candidate.continues_owned_route(0) {
                        let token = candidate.next_producer_round(execution, 0)?;
                        candidate.append_named_successor_publication(
                            execution,
                            token,
                            posture,
                            &none(),
                            creates(keys, occurrence),
                            plan,
                        )?;
                    } else {
                        candidate.append_named_publication(
                            execution,
                            posture,
                            &none(),
                            creates(keys, occurrence),
                            plan,
                        )?;
                    }
                }
                Ok(())
            }),
        )
        .unwrap();
    let result = world.publish(execution, proposal);
    let members = result
        .required_output_publications(&world.installed, execution)
        .flat_map(|origin| {
            origin
                .unwrap()
                .members()
                .map(|member| (member.0.to_owned(), member.3))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    release_test_commit_snapshot(&world.runtime, &result);
    let observation = world.basis().observation();
    world
        .runtime
        .with_persistent_names_at_observation(execution, &observation, |view| {
            members
                .iter()
                .map(
                    |(role, id)| match view.name_of_record(LineageRecordTarget::Entity(*id)) {
                        PersistentNameOfRecord::Live { name, .. } => {
                            (role.clone(), *id, format!("{name:?}"))
                        }
                        other => panic!("{role} is not live under a name: {other:?}"),
                    },
                )
                .collect()
        })
        .unwrap()
}

#[test]
fn only_the_live_rounds_births_execute() {
    let chain = named(&[(&["x", "y"], "round.1"), (&["y", "z"], "round.2")]);
    let one = named(&[(&["y", "z"], "round.2")]);
    let strip = |rows: &[(String, EntityId, String)]| {
        rows.iter()
            .map(|(role, _, name)| (role.clone(), name.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(strip(&chain), strip(&one), "exactly the live round's names");
    let head = named(&[(&["x", "y"], "round.1")]);
    let id = |rows: &[(String, EntityId, String)], role: &str| {
        rows.iter().find(|row| row.0 == role).unwrap().1
    };
    assert_eq!(id(&chain, "y"), id(&head, "y"), "y keeps its identity");
    assert!(
        chain.iter().all(|(role, ..)| role != "x"),
        "no name for the dropped x"
    );
}
