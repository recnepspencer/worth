//! Genuine history records retain the original Fixed leaf and its qualified read.
use super::*;
use crate::application_invariant_acceptance::document_history::model::{
    attach, attach_with_branch, close, with_branch_request, with_request,
};
use crate::application_program::document_history::{HistoryAction, HistoryProgram, HistoryRoot};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::*;
use worth_query_topology_entry::{
    PlanarDocumentHistory, PlanarInitialAdjustment, PlanarOutputDemand, PlanarOutputRead,
};
mod aggregate;
mod journeys;
mod navigation_outputs;
mod profile;
mod race;
mod unavailable;

type History = PlanarDocumentHistory<ConsumerSchema>;
type Runtime = installation::ConsumerRuntime<HistoryProgram>;
type World = installation::ConsumerWorld<HistoryProgram>;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, ConsumerSchema>;
type Document<'a> = WorthQueryApplicationHistoryDocument<'a, ConsumerSchema, History>;
type Edit<'a> = WorthQueryHistoryEdit<
    'a,
    ConsumerSchema,
    History,
    PlanarInitialAdjustment,
    HistoryProgram,
    HistoryAction,
>;
type Finish<'a> = WorthQueryHistoryEditFinish<
    'a,
    ConsumerSchema,
    History,
    PlanarInitialAdjustment,
    HistoryProgram,
    HistoryAction,
>;

fn install() -> World {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, HistoryProgram>::begin()
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
fn warm(application: &Runtime, key: &str) {
    let mut operation = with_request(application, |request| {
        request
            .demand(PlanarOutputDemand::new(key))
            .retain_program_outputs::<HistoryProgram, HistoryRoot>(application)
            .unwrap_or_else(|failure| {
                panic!("read-origin warm construction: {:?}", failure.denial())
            })
    });
    for _ in 0..64 {
        match with_request(application, |request| operation.advance(request)) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                with_request(application, |request| {
                    settled
                        .close(request)
                        .unwrap_or_else(|failure| panic!("warm close: {:?}", failure.denial()))
                });
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("warm read-origin refusal: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("warm read-origin terminal: {:?}", terminal.denial())
            }
        }
    }
    panic!("the existing leaf finishes within the finite fixture traversal")
}
fn begin<'a>(
    document: Document<'a>,
    application: &Runtime,
    key: &str,
    y: u64,
    attempt: u64,
) -> Edit<'a> {
    let input = PlanarInitialAdjustment {
        scope_key: key.into(),
        replacement_y: length(y),
    };
    with_request(application, |request| {
        document
            .edit(input)
            .idempotency_owned(request, attempt)
            .unwrap_or_else(|_| panic!("owned history input capture"))
            .retain_in_program::<HistoryProgram, HistoryAction>(application)
            .unwrap_or_else(|(_, cause)| panic!("history owning construction: {cause:?}"))
    })
}
fn to_selection<'a>(mut edit: Edit<'a>, application: &Runtime) -> Edit<'a> {
    for _ in 0..64 {
        match edit.posture() {
            WorthQueryHistoryEditPosture::AwaitingSelection
            | WorthQueryHistoryEditPosture::AlreadyRecorded
            | WorthQueryHistoryEditPosture::Terminal
            | WorthQueryHistoryEditPosture::Selected => return edit,
            _ => {}
        }
        edit = match with_request(application, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("history phase refusal: {denial:?}")
            }
        };
    }
    panic!("original SDK source/output custody did not reach history selection")
}
fn finish<'a>(mut edit: Edit<'a>, application: &Runtime) -> Finish<'a> {
    for _ in 0..8 {
        match edit.posture() {
            WorthQueryHistoryEditPosture::Selected
            | WorthQueryHistoryEditPosture::AlreadyRecorded
            | WorthQueryHistoryEditPosture::Terminal => {
                return with_request(application, |request| {
                    edit.finish(request)
                        .unwrap_or_else(|(_, cause)| panic!("original history finish: {cause:?}"))
                })
            }
            _ => {}
        }
        edit = match with_request(application, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("history selection: {denial:?}")
            }
        };
    }
    panic!("captured history completion did not select")
}
