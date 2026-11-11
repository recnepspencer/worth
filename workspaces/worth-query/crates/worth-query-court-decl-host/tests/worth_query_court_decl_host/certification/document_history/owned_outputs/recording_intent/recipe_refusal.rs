//! Missing, corrupt and cross-document recipes never select a partial head.
use super::{non_leaf::settle, non_leaf_program as installed, *};
#[derive(Clone, Copy)]
enum Fault {
    Missing,
    Corrupt,
    Foreign,
}
fn exercise(worker: bool, fault: Fault) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, installed::Program> =
        world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request, false);
    let configure = || {
        let document = world::attach(&host);
        host.with_host_advancement(|phase| {
            document
                .with_output_trees::<installed::Program, installed::Tree>(
                    &phase,
                    &request,
                    &host,
                    installed::Demand {
                        source: "a".into(),
                        discovery: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    },
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap()
    };
    let document = configure();
    let before = document.current_step(&request).unwrap().id();
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "record-before-recipe-fault".into())
        .unwrap()
        .retain_in_program::<installed::Program, program::MemberEditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "recording must finish");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                edit = done;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let mut document = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document;
    let selection = document.selection(&request).unwrap();
    let foreign = if matches!(fault, Fault::Foreign) {
        Some(configure())
    } else {
        None
    };
    match fault {
        Fault::Missing => assert!(document.clear_history_discovery_recipes_for_test(before)),
        Fault::Corrupt => assert!(document.corrupt_history_discovery_recipe_for_test(before)),
        Fault::Foreign => {
            let foreign = foreign.as_ref().unwrap();
            assert!(document.replace_history_discovery_recipes_from_for_test(
                before,
                foreign,
                foreign.current_step(&request).unwrap().id()
            ));
        }
    }
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let discovery = installed::observation::Observation::begin();
    let (_, denial) =
        super::super::courts::checkout(&document, &request, before, selection.clone(), 0x1255_f1)
            .unwrap_err();
    let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Preparation(
            WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                WorthQueryApplicationRequestQueryDenial::HistoryOutput(
                    WorthQueryHistoryOutputReadDenial::Authority(cause),
                ),
            ),
        ),
    ) = denial
    else {
        panic!("wrong recipe refusal: {denial:?}")
    };
    match (fault, cause) {
        (Fault::Missing, WorthQueryHistoryNavigationDenial::DiscoveryRecipeMissing { step })
        | (Fault::Corrupt, WorthQueryHistoryNavigationDenial::DiscoveryRecipeCorrupt { step })
        | (
            Fault::Foreign,
            WorthQueryHistoryNavigationDenial::DiscoveryRecipeForeignDocument { step },
        ) => assert_eq!(step, before),
        (_, cause) => panic!("wrong recipe subject: {cause:?}"),
    }
    assert!(
        !discovery.saw("a") && !discovery.saw("b"),
        "corrupt data never reaches installed discovery"
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(document.selection(&request).unwrap(), selection);
    let at = host
        .request(&principal, &scope)
        .on_branch(selection.physical_endpoint().product_branch());
    assert_eq!(
        at.query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        6
    );
    assert_eq!(
        at.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        30
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    if let Some(foreign) = foreign {
        host.with_host_advancement(|phase| foreign.close(&phase).unwrap())
            .unwrap();
    }
}
#[test]
fn a_missing_discovery_recipe_refuses_the_selected_step_without_effect() {
    exercise(false, Fault::Missing);
}
#[test]
fn worker_only_a_missing_discovery_recipe_refuses_the_selected_step_without_effect() {
    exercise(true, Fault::Missing);
}
#[test]
fn a_corrupt_discovery_recipe_refuses_the_selected_step_without_effect() {
    exercise(false, Fault::Corrupt);
}
#[test]
fn worker_only_a_corrupt_discovery_recipe_refuses_the_selected_step_without_effect() {
    exercise(true, Fault::Corrupt);
}
#[test]
fn a_foreign_documents_recipe_refuses_the_selected_step_without_effect() {
    exercise(false, Fault::Foreign);
}
#[test]
fn worker_only_a_foreign_documents_recipe_refuses_the_selected_step_without_effect() {
    exercise(true, Fault::Foreign);
}
