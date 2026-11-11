//! Actual installed root and child external contracts refuse before any source effect.
use super::*;
use crate::application_program::historical::eligibility::{ChildAction, Program, RootAction};
use worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial;

fn install() -> installation::ConsumerWorld<Program> {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap())
        .validated_program()
        .unwrap();
    installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        program,
    )
    .unwrap()
}

macro_rules! refuses_before_effects {
    ($test:ident, $action:ty, $key:expr) => {
        #[test]
        fn $test() {
            let world = install();
            let app = &world.application;
            let document = attach(app, "anchor-a");
            let before = with_request(app, |request| document.current_step(request).unwrap());
            let contact = PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(2),
            };
            reset_planar_source_adjustment_contacts(&contact);
            let mut group = with_request(app, |request| {
                document
                    .group_edits::<Original, Program, $action, 2>(
                        app,
                        request,
                        $key,
                        [
                            (
                                Original(PlanarInitialAdjustment {
                                    scope_key: "anchor-a".into(),
                                    replacement_y: length(2),
                                }),
                                0x9182_5901,
                            ),
                            (
                                Original(PlanarInitialAdjustment {
                                    scope_key: "anchor-a".into(),
                                    replacement_y: length(3),
                                }),
                                0x9182_5902,
                            ),
                        ],
                    )
                    .unwrap()
            });
            with_request(app, |request| {
                let (document, execution) = group.parts();
                assert_eq!(
                    execution.advance(document, request).unwrap(),
                    WorthQueryHistoryGroupPosture::Admitting
                );
                assert!(matches!(
                    execution.advance(document, request),
                    Err(WorthQueryHistoryGroupDenial::History(
                        WorthQueryApplicationHistoryDenial::Document(
                            WorthQueryDocumentHistoryDenial::UnsupportedOperation
                        )
                    ))
                ));
                assert_eq!(execution.completed_members(), 0);
                assert!(execution.completed_source(0).is_none());
                assert!(execution.selected_step().is_none());
                assert!(execution.read_staged(document, request).is_err());
                assert_eq!(document.current_step(request).unwrap().id(), before.id());
                assert_eq!(document.read(request).unwrap().rows()[0].value, length(1));
            });
            assert_eq!(group.posture(), WorthQueryHistoryGroupPosture::Admitting);
            assert_eq!(planar_source_adjustment_contacts(&contact), (0, 0));
            let document = with_request(app, |request| group.abort(request).unwrap());
            close(document, app);
        }
    };
}
refuses_before_effects!(
    external_root_producer_refuses_before_the_group_source_runs,
    RootAction,
    0x9182_5900
);
refuses_before_effects!(
    external_child_producer_refuses_before_the_group_source_runs,
    ChildAction,
    0x9182_5910
);

#[test]
fn unrelated_external_family_does_not_refuse_an_ordinary_fixed_group() {
    let world = install();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let before = with_request(app, |request| document.current_step(request).unwrap());
    let mut group = with_request(app, |request| {
        document
            .group_edits::<Original, Program, FixedAction, 1>(
                app,
                request,
                0x9182_5920,
                [(
                    Original(PlanarInitialAdjustment {
                        scope_key: "anchor-a".into(),
                        replacement_y: length(2),
                    }),
                    0x9182_5921,
                )],
            )
            .unwrap()
    });
    for _ in 0..128 {
        if group.posture() == WorthQueryHistoryGroupPosture::ReadyToComplete {
            break;
        }
        with_request(app, |request| {
            let (document, execution) = group.parts();
            execution.advance(document, request).unwrap();
        });
    }
    assert_eq!(
        group.posture(),
        WorthQueryHistoryGroupPosture::ReadyToComplete
    );
    with_request(app, |request| {
        let (document, execution) = group.parts();
        assert_eq!(document.read(request).unwrap().rows()[0].value, length(1));
        assert_eq!(
            execution
                .completed_source(0)
                .unwrap()
                .settlement()
                .output_count(),
            1
        );
        execution.complete(document, request).unwrap();
    });
    let finished = with_request(app, |request| group.finish(request).unwrap());
    assert_eq!(finished.completions.len(), 1);
    with_request(app, |request| {
        assert_ne!(
            finished.document.current_step(request).unwrap().id(),
            before.id()
        );
        assert_eq!(
            finished.document.read(request).unwrap().rows()[0].value,
            length(3)
        );
    });
    drop(finished.completions);
    close(finished.document, app);
}
