//! Completed original trees qualify one common source before journal adoption.
use super::*;
use crate::application_program::historical::{FixedAction, FixedProgram, Original};
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
    PlanarOutputReadResult,
};

mod branching;
mod group;
mod navigation;
mod race;
type App = installation::ConsumerRuntime<FixedProgram>;
type FullEdit<'a> =
    WorthQueryHistoryEdit<'a, ConsumerSchema, History, Original, FixedProgram, FixedAction>;
type FullFinish<'a> =
    WorthQueryHistoryEditFinish<'a, ConsumerSchema, History, Original, FixedProgram, FixedAction>;

fn install_full() -> installation::ConsumerWorld<FixedProgram> {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, FixedProgram>::begin()
        .with_schema(&ConsumerSchema::declaration().unwrap())
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
fn begin_full<'a>(document: Document<'a>, app: &App, y: u64, key: u64) -> FullEdit<'a> {
    with_request(app, |request| {
        document
            .edit(Original(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(y),
            }))
            .idempotency_owned(request, key)
            .unwrap_or_else(|_| panic!("capture original full-graph intent"))
            .retain_in_program::<FixedProgram, FixedAction>(app)
            .unwrap_or_else(|(_, cause)| panic!("full history construction: {cause:?}"))
    })
}
fn completed<'a>(mut edit: FullEdit<'a>, app: &App) -> FullEdit<'a> {
    for _ in 0..96 {
        if edit.posture() == WorthQueryHistoryEditPosture::AwaitingSelection {
            return edit;
        }
        edit = match with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("full original continuation: {denial:?}")
            }
        };
    }
    panic!("the actual full graph must complete within the finite fixture bound")
}
fn finish_full<'a>(mut edit: FullEdit<'a>, app: &App) -> FullFinish<'a> {
    for _ in 0..8 {
        if edit.posture() == WorthQueryHistoryEditPosture::Selected {
            return with_request(app, |request| {
                edit.finish(request)
                    .unwrap_or_else(|(_, cause)| panic!("full graph finish: {cause:?}"))
            });
        }
        edit = match with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("complete-set qualification/adoption: {denial:?}")
            }
        };
    }
    panic!("qualified common source did not select")
}
fn ordinary(app: &App, branch: WorthQueryProductBranch, y: u64, key: u64) {
    ordinary_for(app, branch, "anchor-a", y, key)
}

fn ordinary_for(app: &App, branch: WorthQueryProductBranch, model: &str, y: u64, key: u64) {
    let mut operation = with_branch_request(app, branch, |request| {
        request
            .mutate(Original(PlanarInitialAdjustment {
                scope_key: model.into(),
                replacement_y: length(y),
            }))
            .without_source()
            .idempotency_owned(key)
            .unwrap_or_else(|(_, _, cause)| panic!("outside input: {cause:?}"))
            .retain_operation::<FixedProgram, FixedAction>(app)
            .unwrap_or_else(|(_, cause)| panic!("outside owner: {cause:?}"))
    });
    for _ in 0..96 {
        match with_branch_request(app, branch, |request| operation.advance(request)) {
            WorthQueryOperationAdvance::Progress {
                operation: next, ..
            } => operation = next,
            WorthQueryOperationAdvance::Settled(settled) => {
                let delivery = with_branch_request(app, branch, |request| {
                    settled
                        .close_for_delivery(request)
                        .unwrap_or_else(|(_, cause)| panic!("outside close: {cause:?}"))
                });
                drop(delivery);
                return;
            }
            _ => panic!("outside original Fixed tree must settle"),
        }
    }
    panic!("outside tree did not finish")
}

#[test]
fn completed_output_graph_records_its_common_basis_and_retains_old_dependent_values() {
    let world = install_full();
    let app = &world.application;
    let mut document = attach(app, "anchor-a");
    let untouched = attach(app, "remote-a");
    let unchanged = with_request(app, |request| {
        untouched.read(request).unwrap().rows().to_vec()
    });
    let mut first = None;
    for (index, (y, child, expected_child)) in [
        (2, "remote-b", 43),
        (3, "sibling-b", 23),
        (5, "sibling-c", 32),
    ]
    .into_iter()
    .enumerate()
    {
        let contacts = PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(y),
        };
        reset_planar_source_adjustment_contacts(&contacts);
        let edit = completed(
            begin_full(document, app, y, 0x9182_e100 + index as u64),
            app,
        );
        let original = edit.completed_source().unwrap();
        let settlement = original.settlement();
        assert_eq!(
            settlement.output_count(),
            1,
            "the declared dependent actually completed"
        );
        let source = settlement.performed_source().unwrap();
        let a = source.root_fulfillment().source_commit().clone();
        let b = source.root_fulfillment().output_commit().clone();
        let c = settlement.observation().selected_commit().clone();
        assert_ne!(a, b);
        assert_ne!(
            b, c,
            "the final dependent publication follows the actual root"
        );
        assert_eq!(
            with_branch_request(app, original.branch(), |request| request
                .at(settlement.observation())
                .query(PlanarRead {
                    body_key: format!("final:{child}")
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y),
            length(expected_child)
        );
        let finished = finish_full(edit, app);
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = finished.outcome
        else {
            panic!("complete original tree must record")
        };
        assert_eq!(step.endpoint().selected_commit(), &c);
        assert_ne!(
            step.endpoint().selected_commit(),
            &b,
            "root-only fulfillment cannot select the aggregate endpoint"
        );
        document = finished.document;
        assert_eq!(
            with_request(app, |request| document
                .read(request)
                .unwrap()
                .rows()
                .to_vec()),
            vec![PlanarOutputReadResult {
                body_key: "anchor-a".into(),
                successor_body_key: "anchor-b".into(),
                value: length(y + 1)
            }]
        );
        assert_eq!(
            with_request(app, |request| untouched
                .read(request)
                .unwrap()
                .rows()
                .to_vec()),
            unchanged
        );
        assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
        if index == 0 {
            first = Some((step, delivery));
        } else {
            drop(delivery);
        }
    }
    let (step, delivery) = first.unwrap();
    let before = [2, 3, 5].map(|y| {
        planar_source_adjustment_contacts(&PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(y),
        })
    });
    let preview = with_request(app, |request| document.preview(request, step.id()).unwrap());
    with_request(app, |request| {
        let old = preview.read(&document, request).unwrap();
        assert_eq!(
            old.rows(),
            &[PlanarOutputReadResult {
                body_key: "anchor-a".into(),
                successor_body_key: "anchor-b".into(),
                value: length(3)
            }]
        );
        assert_eq!(old.source(), step.endpoint());
        assert_eq!(
            old.required_root_posture(),
            Some(WorthQueryOutputSettlementPosture::Performed)
        );
        old.close();
    });
    with_branch_request(app, delivery.branch(), |request| {
        let dependent = request
            .at(delivery.settlement().observation())
            .query(PlanarRead {
                body_key: "final:remote-b".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            dependent.rows(),
            &[worth_query_topology_entry::PlanarReadResult {
                body_key: "final:remote-b".into(),
                y: length(43)
            }]
        );
    });
    let current = with_request(app, |request| document.current_step(request).unwrap());
    assert_eq!(
        with_branch_request(app, current.endpoint().product_branch(), |request| request
            .query(PlanarRead {
                body_key: "final:sibling-c".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y),
        length(32)
    );
    assert_eq!(
        [2, 3, 5].map(
            |y| planar_source_adjustment_contacts(&PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(y)
            })
        ),
        before,
        "historical reads run no source command"
    );
    preview.close();
    drop(delivery);
    close(untouched, app);
    close(document, app);
}
