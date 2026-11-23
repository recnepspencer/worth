//! Tracked `name_of` and `resolve` over the runtime-maintained derived name
//! index: exact, plural, side and carrier selections, retirement, unknown
//! names and kind refusals, each at the reader's pinned basis.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, ExecutionNameResolveDenial, NameReadDependencies,
};

use crate::naming_execution_fixture::{
    execution_lease_for_workers, CadNamingSchema, Edge, Face, FaceSplitDisc,
};
use crate::naming_tracked_world::{NamedFace, TrackedWorld};

fn in_name_order(faces: &[NamedFace]) -> Vec<NamedFace> {
    let mut ordered = faces.to_vec();
    ordered.sort_by(|a, b| a.name.cmp(&b.name));
    ordered
}

#[test]
fn entity_resolution_refuses_ambiguity_and_follows_split_merge_and_retirement() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let deps = &mut NameReadDependencies::new();
    let resolve = |observation, held: &NamedFace, selection: &NameSelection| {
        world
            .reader(&execution, observation)
            .resolve_entity::<CadNamingSchema, Face>(
                &held.name,
                selection,
                &mut NameReadDependencies::new(),
            )
    };

    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let f1 = seeded[0];
    let before_split = world.head();
    let reader = world.reader(&execution, &before_split);
    assert_eq!(
        reader.entity_name_of::<CadNamingSchema, Face>(f1.id, deps),
        Ok(f1.name)
    );
    assert!(matches!(
        reader.entity_name_of::<CadNamingSchema, Edge>(f1.id, deps),
        Err(ExecutionNameReadDenial::KindIdentityMismatch { .. })
    ));
    assert_eq!(
        resolve(&before_split, &f1, &NameSelection::Exact),
        Ok(ResolvedName::Exact(f1.id))
    );

    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    let after_split = world.head();
    let ordered = in_name_order(&pieces);
    assert_eq!(
        resolve(&after_split, &f1, &NameSelection::Exact),
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::Ambiguous {
                candidates: ordered.iter().map(|face| face.name).collect(),
            }
        )),
        "Exact on two live successors refuses with both; it never picks a least member"
    );
    assert_eq!(
        resolve(&after_split, &f1, &NameSelection::AllDescendants),
        Ok(ResolvedName::Plural(
            ordered.iter().map(|face| face.id).collect()
        ))
    );
    let side_two = NameSelection::keep_side(&FaceSplitDisc::Side { side: 2 }).unwrap();
    assert_eq!(
        resolve(&after_split, &f1, &side_two),
        Ok(ResolvedName::Exact(b.id))
    );
    let bounded = NameSelection::bounded_by_carriers([a.name.erased()]).unwrap();
    assert_eq!(
        resolve(&after_split, &f1, &bounded),
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::NoMatchingCandidate {
                live_candidates: ordered.iter().map(|face| face.name).collect(),
            }
        )),
        "entities declare no carriers, so a carrier bound matches none and says so"
    );
    assert_eq!(
        world
            .reader(&execution, &after_split)
            .entity_name_of::<CadNamingSchema, Face>(f1.id, deps),
        Err(ExecutionNameReadDenial::RecordNoLongerNamed {
            record: worth_relational::facade::lineage::LineageRecordTarget::Entity(f1.id),
        })
    );
    assert_eq!(
        resolve(&before_split, &f1, &NameSelection::Exact),
        Ok(ResolvedName::Exact(f1.id)),
        "an older pinned basis keeps its answer"
    );

    world.merge_keeping(&execution, a, b);
    let after_merge = world.head();
    assert_eq!(
        resolve(&after_merge, &b, &NameSelection::Exact),
        Ok(ResolvedName::Exact(a.id))
    );
    assert_eq!(
        resolve(&after_merge, &f1, &NameSelection::Exact),
        Ok(ResolvedName::Exact(a.id))
    );

    let retired = world.retire(&execution, a);
    let after_retire = world.head();
    let at_commit = retired.commit.commit_id.0;
    assert_eq!(
        resolve(&after_retire, &f1, &NameSelection::Exact),
        Ok(ResolvedName::Retired { at_commit })
    );
    assert_eq!(
        resolve(&after_retire, &b, &NameSelection::AllDescendants),
        Ok(ResolvedName::Retired { at_commit })
    );

    let elsewhere = TrackedWorld::new();
    let (foreign, _) = elsewhere.seed(&execution, "occ.elsewhere", &["z"]);
    assert_eq!(
        resolve(&after_retire, &foreign[0], &NameSelection::Exact),
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::UnknownName
        )),
        "a name born only in another runtime is unknown here"
    );
}

#[test]
fn relation_names_resolve_and_retire_under_their_own_kind() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (faces, _) = world.seed(&execution, "occ.ends", &["e1", "e2"]);
    let (seam, _) = world.seam(&execution, "occ.seam", faces[0], faces[1]);
    let live = world.head();
    let deps = &mut NameReadDependencies::new();
    let reader = world.reader(&execution, &live);
    assert_eq!(
        reader.relation_name_of::<crate::naming_execution_fixture::Seam>(seam.id, deps),
        Ok(seam.name)
    );
    assert_eq!(
        reader.resolve_relation(&seam.name, &NameSelection::Exact, deps),
        Ok(ResolvedName::Exact(seam.id))
    );
    let retired = world.retire_seam(&execution, seam);
    let gone = world.head();
    let reader = world.reader(&execution, &gone);
    assert_eq!(
        reader.resolve_relation(&seam.name, &NameSelection::Exact, deps),
        Ok(ResolvedName::Retired {
            at_commit: retired.commit.commit_id.0
        })
    );
    assert!(matches!(
        reader.relation_name_of::<crate::naming_execution_fixture::Seam>(seam.id, deps),
        Err(ExecutionNameReadDenial::RecordNoLongerNamed { .. })
    ));
}
