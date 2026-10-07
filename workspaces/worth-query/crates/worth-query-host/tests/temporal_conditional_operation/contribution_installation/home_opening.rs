//! One open call per entry: which homes each entry starts, resumes or refuses.
use super::checkpoint_transition::{configuration, home_of};
use super::*;
use application_installation::{
    ApplicationHome, WorthQueryApplicationOpenDenial as Denial, WorthQueryHomeAbsent,
    WorthQueryHomeForm, WorthQueryHomeOpening as Opening, WorthQueryOpenEntryKind as EntryKind,
    WorthQueryRefusedHome as RefusedHome, WorthQueryReopenDeferral, WorthQueryReturnPoint,
    WorthQueryStateOwner,
};

fn seed(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    installed: &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    let principal = installed
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    seed_graph(graph, &principal, "home-opening", 0, 1, true);
    Ok(())
}

fn declaration_open<'open>(
) -> application_installation::WorthQueryDeclarationOpen<'open, TemporalHostSchema> {
    application_installation::declaration(
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
}

fn program_open<'open>() -> application_installation::WorthQueryProgramOpen<
    'open,
    TemporalHostSchema,
    TemporalInstallationProgram,
> {
    application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
}

#[test]
fn a_program_home_starts_once_and_resumes_without_reseeding() {
    let mut seed_calls = 0;
    let started = program_open()
        .initial_state(|graph, installed| {
            seed_calls += 1;
            seed(graph, installed)
        })
        .open(ApplicationHome::memory())
        .expect("an empty home starts with the declared initial state");
    assert_eq!(started.opening(), &Opening::Started);
    assert_eq!(seed_calls, 1, "the empty-home hook must run exactly once");
    assert_seed_record(&started);
    let image = started.capture_application_checkpoint().unwrap();
    drop(started);
    let mut reseeded = false;
    let resumed = program_open()
        .initial_state(|_, _| {
            reseeded = true;
            Ok(())
        })
        .open(home_of(&image))
        .expect("a home holding this program's image resumes");
    assert_eq!(
        resumed.opening(),
        &Opening::Resumed {
            installed: *validated_program().revision()
        }
    );
    assert_seed_record(&resumed);
    drop(resumed);
    assert!(!reseeded, "a resumed home must skip the initial state");
}

#[test]
fn each_entry_refuses_the_image_the_other_entry_wrote() {
    let program_image = program_open()
        .initial_state(seed)
        .open(ApplicationHome::memory())
        .unwrap()
        .capture_application_checkpoint()
        .unwrap();
    let declaration_image = declaration_open()
        .initial_state(seed)
        .open(ApplicationHome::memory())
        .expect("the declaration entry starts an empty home")
        .capture_application_checkpoint()
        .unwrap();

    let refusal = declaration_open()
        .open(home_of(&program_image))
        .err()
        .expect("the declaration entry must refuse a program image");
    assert!(matches!(
        refusal.denial,
        Denial::EntryKindMismatch {
            image: EntryKind::Program,
            entry: EntryKind::Declaration,
        }
    ));
    let home = match refusal.home {
        RefusedHome::Unchanged(home) => home,
        other => panic!("a mismatch must leave the home unchanged: {other:?}"),
    };
    program_open()
        .open(home)
        .expect("the refused home still resumes under its own entry");

    let refusal = program_open()
        .open(home_of(&declaration_image))
        .err()
        .expect("the program entry must refuse a declaration image");
    assert!(matches!(
        refusal.denial,
        Denial::EntryKindMismatch {
            image: EntryKind::Declaration,
            entry: EntryKind::Program,
        }
    ));
    let home = match refusal.home {
        RefusedHome::Unchanged(home) => home,
        other => panic!("a mismatch must leave the home unchanged: {other:?}"),
    };
    declaration_open()
        .open(home)
        .expect("the refused home still resumes under its own entry");
}

#[test]
fn a_path_home_is_refused_with_its_deferral_and_nothing_is_created() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("deferred-application-home");
    let absent = WorthQueryHomeAbsent {
        form: WorthQueryHomeForm::At,
        deferral: WorthQueryReopenDeferral {
            owner: WorthQueryStateOwner::Relational,
            return_point: WorthQueryReturnPoint::RelationalOnStore,
        },
    };
    let mut seeded = false;
    let refusal = program_open()
        .initial_state(|_, _| {
            seeded = true;
            Ok(())
        })
        .open(ApplicationHome::at(&path))
        .err()
        .expect("the program entry cannot open a path home yet");
    assert!(matches!(refusal.denial, Denial::Home(found) if found == absent));
    assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
    assert!(!seeded, "a refused home must not run the initial state");
    let refusal = declaration_open()
        .open(ApplicationHome::at(&path))
        .err()
        .expect("the declaration entry cannot open a path home yet");
    assert!(matches!(refusal.denial, Denial::Home(found) if found == absent));
    assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
    assert!(!path.exists(), "a refused path home must create nothing");
}

fn assert_seed_record(application: &super::checkpoint_transition::Target) {
    let selected = application
        .on_branch(application.current_world())
        .select()
        .unwrap();
    let _intent = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the first seed's intent must remain visible");
    // The accessor must work on the graph installed by this open.
    let authority = application
        .runtime()
        .retain_invariant_projection_authority();
    let projection = authority
        .project(|reader| {
            let entity = reader
                .resolve_entity(IntentIdentityField::reference(), "intent-1".to_string())
                .unwrap();
            reader.field(&entity, IntentGateField::reference())
        })
        .unwrap();
    assert_eq!(projection.into_parts().0, Some("home-opening".to_string()));
}
