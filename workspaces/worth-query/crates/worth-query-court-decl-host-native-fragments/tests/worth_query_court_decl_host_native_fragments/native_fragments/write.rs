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
            view.write(&item, RequiredNote, &OptionalText { text: None }, work)
                .unwrap();
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

#[test]
fn an_all_none_case_has_native_presence_after_selection() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| view.write(&item, Noted, &OptionalText { text: None }, work)).unwrap();
    }
    committed(&world, candidate, 85);
    assert_eq!(
        world.read().unwrap(),
        FormValue::Noted(OptionalText { text: None })
    );
}

#[test]
fn an_all_none_case_has_native_presence_at_birth() {
    let world = World::new(Mode::Named);
    let (mut candidate, _) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.create(Item, "none-at-birth").unwrap();
        work(|work| {
            view.write(&item, ItemIdentity, &PrincipalData { id: 37 }, work)
                .unwrap();
            view.write(&item, Metrics, &measured(), work).unwrap();
            view.write(&item, RequiredNote, &OptionalText { text: None }, work)
                .unwrap();
            view.write(&item, Noted, &OptionalText { text: None }, work)
                .unwrap();
        });
    }
    committed(&world, candidate, 86);
    assert_eq!(
        world.inspect(37).unwrap(),
        FormValue::Noted(OptionalText { text: None })
    );
}

#[test]
fn replacing_a_created_optional_record_removes_the_previous_value() {
    let world = World::new(Mode::Named);
    let (mut candidate, _) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.create(Item, "replaced-note").unwrap();
        work(|work| {
            view.write(&item, ItemIdentity, &PrincipalData { id: 41 }, work)
                .unwrap();
            view.write(&item, Metrics, &measured(), work).unwrap();
            view.write(&item, RequiredNote, &OptionalText { text: None }, work)
                .unwrap();
            view.write(
                &item,
                Named,
                &Caption {
                    text: "named".to_owned(),
                },
                work,
            )
            .unwrap();
            view.write(
                &item,
                Note,
                &OptionalText {
                    text: Some("first".to_owned()),
                },
                work,
            )
            .unwrap();
            view.write(&item, Note, &OptionalText { text: None }, work)
                .unwrap();
        });
    }
    committed(&world, candidate, 87);
    assert_eq!(
        world.inspect_note(41).unwrap(),
        Some(OptionalText { text: None })
    );
}

#[test]
fn a_record_write_cannot_change_an_existing_read_only_field() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        let denied = work(|work| view.write(&item, ItemIdentity, &PrincipalData { id: 99 }, work));
        assert!(
            matches!(denied, Err(FragmentDenial::ReadOnly { path }) if path == "fixture.item.identity.id")
        );
    }
    committed(&world, candidate, 88);
    assert_eq!(
        world.read().unwrap(),
        FormValue::Named(Caption {
            text: "initial".to_owned()
        })
    );
}

#[test]
fn an_optional_case_value_and_its_presence_are_committed_together() {
    let world = World::new(Mode::Named);
    let (mut candidate, target) = world.candidate(ChangeForm::reference());
    let value = OptionalText {
        text: Some("note".to_owned()),
    };
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.target(&target).unwrap();
        work(|work| view.write(&item, Noted, &value, work)).unwrap();
    }
    committed(&world, candidate, 89);
    assert_eq!(world.read().unwrap(), FormValue::Noted(value));
}

#[test]
fn column_presence_changes_retain_the_sdk_parent_in_the_same_commit() {
    let world = World::new(Mode::Named);
    for (key, text) in [(90, Some("column".to_owned())), (91, None)] {
        let (mut candidate, target) = world.column_candidate();
        candidate
            .write_optional_field(
                &target,
                select_field(Fixture, Item, Note, "text"),
                text.clone(),
            )
            .unwrap();
        committed(&world, candidate, key);
        assert_eq!(world.inspect_note(11).unwrap(), Some(OptionalText { text }));
    }
}
#[test]
fn creation_retains_explicit_presence_for_a_required_record_with_all_optional_values() {
    let world = World::new(Mode::Named);
    let (mut candidate, _) = world.candidate(ChangeForm::reference());
    {
        let mut view = world
            .application
            .fragment_candidate(&mut candidate, Fixture)
            .unwrap();
        let item = view.create(Item, "explicit-required-note").unwrap();
        work(|work| {
            view.write(&item, ItemIdentity, &PrincipalData { id: 47 }, work)
                .unwrap();
            view.write(&item, Metrics, &measured(), work).unwrap();
            view.write(&item, RequiredNote, &OptionalText { text: None }, work)
                .unwrap();
            view.write(
                &item,
                Named,
                &Caption {
                    text: "created".into(),
                },
                work,
            )
            .unwrap();
        });
    }
    committed(&world, candidate, 92);
    assert_eq!(
        world.inspect_required_note(47).unwrap(),
        OptionalText { text: None }
    );
}
