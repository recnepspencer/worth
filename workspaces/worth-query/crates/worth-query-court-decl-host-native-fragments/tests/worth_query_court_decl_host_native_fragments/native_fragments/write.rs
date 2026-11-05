use super::*;

fn committed<O: ApplicationOperationMarkerIdentity<Fixture> + 'static>(
    world: &World,
    candidate: world::Program<O>,
    key: u8,
) {
    let outcome = world.application.compare_and_commit_application(
        candidate.finish().unwrap(),
        WorthQueryApplicationIdempotencyBinding::for_host_commit::<Fixture, O, _, _>(&key, &key)
            .unwrap(),
        SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "{outcome:?}"
    );
}

#[test]
fn writing_a_case_clears_its_native_siblings_in_the_same_commit() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| view.write(&item, Measured, &measured(), work)).unwrap();
    }
    committed(&world, candidate, 81);
    assert_eq!(world.read().unwrap(), FormValue::Measured(measured()));
}

#[test]
fn repeated_case_selection_reuses_only_its_own_pending_clears() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| {
            view.write(&item, Measured, &measured(), work).unwrap();
            view.write(
                &item,
                Named,
                &Caption {
                    text: "final".to_owned(),
                },
                work,
            )
            .unwrap();
        });
    }
    committed(&world, candidate, 82);
    assert_eq!(
        world.read().unwrap(),
        FormValue::Named(Caption {
            text: "final".to_owned()
        })
    );
}

#[test]
fn child_field_authority_cannot_clear_a_sibling_and_retains_no_effect_prefix() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(FieldsOnly::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        assert!(matches!(
            work(|work| view.write(
                &item,
                Named,
                &Caption {
                    text: "forbidden".to_owned()
                },
                work
            )),
            Err(FragmentDenial::Write(_))
        ));
    }
    committed(&world, candidate, 84);
    assert_eq!(
        world.read().unwrap(),
        FormValue::Named(Caption {
            text: "initial".to_owned()
        })
    );
}

#[test]
fn an_application_cannot_open_a_foreign_candidates_fragment_view() {
    let first = World::new(Mode::Named);
    let second = World::new(Mode::Named);
    let (mut candidate, _) = first.candidate(ChangeForm::reference());
    assert!(
        matches!(second.application.fragment_candidate(&mut candidate, Fixture), Err(FragmentDenial::Write(denial)) if denial.kind() == WorthQueryApplicationAttemptDenialKind::ForeignApplication)
    );
}

#[test]
fn created_case_selection_removes_the_previous_cases_fields_before_birth() {
    let world = World::new(Mode::Named);
    let (mut candidate, _) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.create(Item, "created-item").unwrap();
        work(|work| {
            view.write(&item, ItemIdentity, &PrincipalData { id: 29 }, work)
                .unwrap();
            view.write(&item, Metrics, &measured(), work).unwrap();
            view.write(&item, Measured, &measured(), work).unwrap();
            view.write(
                &item,
                Named,
                &Caption {
                    text: "born".to_owned(),
                },
                work,
            )
            .unwrap();
        });
    }
    committed(&world, candidate, 83);
    assert_eq!(
        world.inspect(29).unwrap(),
        FormValue::Named(Caption {
            text: "born".to_owned()
        })
    );
}
