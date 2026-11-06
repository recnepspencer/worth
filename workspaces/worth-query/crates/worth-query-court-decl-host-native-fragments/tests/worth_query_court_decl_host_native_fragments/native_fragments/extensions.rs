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
