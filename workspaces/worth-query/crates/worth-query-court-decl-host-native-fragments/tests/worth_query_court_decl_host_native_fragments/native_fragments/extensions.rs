//! Two features read and write the same Native entity through ordinary fragment Views.
use super::*;
use worth_query_testkit_decl::fragment_extension_fixture::*;

fn inspect(world: &World) -> (Option<OptionalText>, Option<Caption>) {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_operation::<ChangeForm, _>(
                    &phase,
                    |reader| {
                        let item = reader
                            .fragment(Fixture)
                            .unwrap()
                            .resolve(
                                record_field(Fixture, Item, ItemIdentity, "id").unwrap(),
                                11u64,
                            )
                            .unwrap();
                        let annotation = work(|work| {
                            reader
                                .fragment(Annotations)
                                .unwrap()
                                .read(&item, Annotation, work)
                        })
                        .unwrap();
                        let case = work(|work| {
                            reader
                                .fragment(Classifications)
                                .unwrap()
                                .read(&item, Classified, work)
                        })
                        .unwrap();
                        (annotation, case)
                    },
                    SystemAllocation,
                )
                .unwrap()
                .into_output()
        })
        .unwrap()
}

#[test]
fn extension_views_read_write_presence_and_an_optional_one_of_case() {
    let world = World::new(Mode::Named);
    assert_eq!(inspect(&world), (None, None));
    let (mut candidate, target, _) =
        world.candidate_observing(ChangeForm::reference(), false, true);
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Annotations)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| view.write(&item, Annotation, &OptionalText { text: None }, work)).unwrap();
    }
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Classifications)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| {
            view.write(
                &item,
                Classified,
                &Caption {
                    text: "selected".to_owned(),
                },
                work,
            )
        })
        .unwrap();
    }
    super::write::committed(&world, candidate, 101);
    assert_eq!(
        inspect(&world),
        (
            Some(OptionalText { text: None }),
            Some(Caption {
                text: "selected".to_owned()
            })
        )
    );
}

#[test]
fn extension_relation_links_and_traverses_through_view() {
    let world = World::new(Mode::Named);
    let (mut candidate, target, _) =
        world.candidate_observing(ChangeForm::reference(), false, true);
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Annotations)
            .unwrap();
        let item = view.target(&target).unwrap();
        view.link(&item, Annotates, &item).unwrap();
    }
    super::write::committed(&world, candidate, 102);
    let result = world
        .application
        .with_host_advancement(|phase| {
            world.invariant.project_operation::<ChangeForm, _>(
                &phase,
                |reader| {
                    let item = reader
                        .fragment(Fixture)
                        .unwrap()
                        .resolve(
                            record_field(Fixture, Item, ItemIdentity, "id").unwrap(),
                            11u64,
                        )
                        .unwrap();
                    let targets = reader
                        .fragment(Annotations)
                        .unwrap()
                        .relations_from(&item, Annotates)
                        .unwrap();
                    assert_eq!(targets.len(), 1);
                    let row = work(|work| {
                        reader
                            .fragment(Fixture)
                            .unwrap()
                            .read(&targets[0], Named, work)
                    })
                    .unwrap();
                    assert_eq!(row.unwrap().text, "initial");
                },
                SystemAllocation,
            )
        })
        .unwrap();
    result.unwrap();
}

#[test]
fn extension_relation_view_traversal_requires_the_declared_read_grant() {
    let world = World::new(Mode::Named);
    world
        .admitted_projection(FieldsOnly::reference(), |reader, item| {
            assert!(matches!(
                reader
                    .fragment(Annotations)
                    .unwrap()
                    .relations_from(item, Annotates),
                Err(FragmentDenial::Read(_))
            ));
        })
        .unwrap();
}

#[test]
fn a_swallowed_extension_relation_traversal_failure_cannot_seal_a_projection() {
    let foreign = World::new(Mode::Named);
    let item = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(select_field(Fixture, Item, ItemIdentity, "id"), 11u64)
                            .unwrap()
                            .for_fragment()
                    },
                    SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let world = World::new(Mode::Named);
    let result = world.admitted_projection(ChangeForm::reference(), |reader, _| {
        assert!(reader
            .fragment(Annotations)
            .unwrap()
            .relations_from(&item, Annotates)
            .is_err());
    });
    assert!(
        result.is_err(),
        "failed adjacency observation must block sealing even when swallowed"
    );
}
