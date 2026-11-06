//! A real discovery READ policy remains distinct from model READ and NAV.
use super::*;
use crate::application_program::historical::owner_discovery::{Action, Program, Root};
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryApplicationQueryAdmissionDenialKind as QueryAdmission,
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    },
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    },
};
use worth_query_topology_entry::{OwnerDiscoveryRead, PlanarRead};

type App = installation::ConsumerRuntime<Program>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Actor {
    Owner,
    Sibling,
}
impl Actor {
    fn credential(self) -> authentication::LocalCredential {
        match self {
            Self::Owner => authentication::LocalCredential::issued_for_model_owner(),
            Self::Sibling => authentication::LocalCredential::issued_for_sibling_owner(),
        }
    }
}
fn request_as<T>(
    app: &App,
    actor: Actor,
    branch: WorthQueryProductBranch,
    body: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(app.installed_schema());
    let scope = authentication::request_scope();
    let principal =
        authentication::block_on(adapter.authenticate(actor.credential(), &scope)).unwrap();
    body(&app.request(&principal, &scope).on_branch(branch))
}
fn install_guarded() -> installation::ConsumerWorld<Program> {
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
fn record_current<'a>(document: Document<'a>, app: &App) -> Document<'a> {
    let mut edit = with_request(app, |request| {
        document
            .edit(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(5),
            })
            .idempotency_owned(request, 0x9182_f001)
            .unwrap()
            .retain_in_program::<Program, Action>(app)
            .unwrap_or_else(|(_, cause)| panic!("declared guarded action: {cause:?}"))
    });
    for _ in 0..96 {
        if edit.posture() == WorthQueryHistoryEditPosture::Selected {
            let finished = with_request(app, |request| edit.finish(request))
                .unwrap_or_else(|(_, cause)| panic!("guarded history finish: {cause:?}"));
            let WorthQueryFixedHistoryEditOutcome::Recorded { delivery, .. } = finished.outcome
            else {
                panic!("record a genuine current future")
            };
            drop(delivery);
            return finished.document;
        }
        edit = match with_request(app, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("guarded source: {denial:?}")
            }
        };
    }
    panic!("guarded source did not select");
}
fn denied_read(cause: &WorthQueryHistoryNavigationAdvanceDenial) {
    let WorthQueryHistoryNavigationAdvanceDenial::Outputs(
        WorthQueryHistoryOutputDenial::Preparation(
            WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                WorthQueryApplicationRequestQueryDenial::Admission(denial),
            ),
        ),
    ) = cause
    else {
        panic!("actual discovery READ admission refusal: {cause:?}")
    };
    assert_eq!(
        denial.kind(),
        QueryAdmission::Authorization(Authorization::Refused(Refusal::PermissionDenied))
    );
    assert_eq!(denial.subject(), "OwnerDiscoveryPolicy");
}
#[test]
fn historical_discovery_requires_its_own_read_ability_and_preserves_original_custody() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for actor in [Actor::Sibling, Actor::Owner] {
        let world = install_guarded();
        let app = &world.application;
        let document = attach(app, "anchor-a");
        let target = with_request(app, |request| document.current_step(request).unwrap().id());
        let document = record_current(document, app);
        let branch = with_request(app, |request| document.selection(request).unwrap())
            .physical_endpoint()
            .product_branch();
        let document = app
            .with_host_advancement(|phase| {
                request_as(app, actor, branch, |request| {
                    document
                        .with_outputs::<Program, Root>(
                            &phase,
                            request,
                            app,
                            PlanarOutputDemand::new("anchor-a"),
                            Default::default(),
                        )
                        .unwrap_or_else(|(_, cause)| panic!("original guarded profile: {cause:?}"))
                })
            })
            .unwrap();
        let before = request_as(app, actor, branch, |request| {
            assert_eq!(document.read(request).unwrap().rows()[0].value, length(6));
            document.selection(request).unwrap()
        });
        let mut navigation = request_as(app, actor, branch, |request| {
            document
                .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
                .checkout(target)
                .expected_selection(before.clone())
                .idempotency(0x9182_f002)
                .prepare(request)
                .unwrap()
        });
        let contacts = app.producer_contacts_on_this_thread_for_test();
        let mut finished = false;
        for _ in 0..96 {
            match request_as(app, actor, branch, |request| navigation.advance(request)) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => navigation = next,
                WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                    assert_eq!(actor, Actor::Sibling, "{cause:?}");
                    denied_read(&cause);
                    assert_eq!(
                        owner.posture(),
                        WorthQueryHistoryNavigationPosture::AwaitingOutputs
                    );
                    let candidate = owner.candidate_branch_for_test().unwrap();
                    assert_eq!(
                        app.producer_contacts_on_this_thread_for_test(),
                        contacts + 1,
                        "the public root ran; discovery refusal prevents the child"
                    );
                    assert_eq!(
                        request_as(app, actor, branch, |request| document
                            .selection(request)
                            .unwrap()),
                        before
                    );
                    assert_eq!(
                        request_as(app, actor, branch, |request| document
                            .read(request)
                            .unwrap()
                            .rows()[0]
                            .value),
                        length(6)
                    );
                    assert_eq!(
                        request_as(app, actor, candidate, |request| request
                            .query(PlanarOutputRead {
                                body_key: "anchor-a".into(),
                            })
                            .execute()
                            .unwrap()
                            .rows()[0]
                            .value),
                        length(2)
                    );
                    request_as(app, actor, branch, |request| owner.discard(request))
                        .unwrap_or_else(|(_, cause)| {
                            panic!("original denied discovery disposal: {cause:?}")
                        });
                    let refusal = app
                        .with_host_advancement(|phase| {
                            match app.on_branch(candidate).select(&phase) {
                                Err(cause) => cause,
                                Ok(_) => panic!("the actual original candidate must retire"),
                            }
                        })
                        .unwrap();
                    assert_eq!(
                        refusal,
                        WorthQueryProductBranchAdmissionDenial::Refused(
                            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
                        )
                    );
                    finished = true;
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Selected(done) => {
                    assert_eq!(actor, Actor::Owner);
                    assert_eq!(done.requested_node(), target);
                    assert_ne!(
                        done.selection().physical_endpoint(),
                        before.physical_endpoint()
                    );
                    assert_eq!(
                        app.producer_contacts_on_this_thread_for_test(),
                        contacts + 2
                    );
                    assert_eq!(
                        request_as(app, actor, branch, |request| document
                            .read(request)
                            .unwrap()
                            .rows()[0]
                            .value),
                        length(2)
                    );
                    let selected = done.selection().physical_endpoint().product_branch();
                    // The retained root Y1 chooses remote-b, whose authored Y41
                    // produces the independent final value43 in complete E.
                    assert_eq!(
                        request_as(app, actor, selected, |request| request
                            .query(PlanarRead {
                                body_key: "final:remote-b".into(),
                            })
                            .execute()
                            .unwrap()
                            .rows()[0]
                            .y),
                        length(43)
                    );
                    assert_eq!(
                        request_as(app, actor, selected, |request| request
                            .query(OwnerDiscoveryRead {
                                body_key: "anchor-a".into(),
                            })
                            .execute()
                            .unwrap()
                            .rows()[0]
                            .y),
                        length(1)
                    );
                    finished = true;
                    break;
                }
            }
        }
        assert!(
            finished,
            "the guarded tree must finish or return its exact refusal"
        );
        app.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
    }
}
