//! One original source owner recovers the actual interrupted combined correction.
pub mod unwind;
use super::{
    authentication::*,
    composite_definition_court as composite,
    composite_definition_handler::handler_contacts,
    composite_definition_model::*,
    host,
    program::Program,
    query::Row,
    required_output_courts::{head, limits},
    schema::NumericSchema,
};
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::*,
    primary_graph::WorthQueryConditionalSignalDecision, runtime::ProductUnpublishedCause,
};
type Action = ApplicationActionReference<
    NumericSchema,
    ApplicationRootComposition,
    Feature,
    Binding,
    Completion,
>;

pub fn run() {
    let mut application = host::install();
    exercise(&application);
    application.close_conditional_runtime().unwrap();
    println!("original combined correction owner recovers, replays and closes without reexecuting its handler");
}

fn exercise(application: &host::Host) {
    let mut source = composite::change(application);
    composite::deliver(application, &mut source);
    assert_eq!(
        composite::observe(application, WorthQueryConditionalSignalDecision::Suppressed),
        0
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let selected = request.corrections().select(&source, limits()).unwrap();
    let before = handler_contacts();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = request
        .corrections()
        .reverse(&selected)
        .idempotency("combined-recovery")
        .limits(limits())
        .prepare_operation::<CorrectDefinition, Program, Action>(application)
        .unwrap()
    else {
        panic!("the recorded combined action prepares");
    };
    let prepared_contacts = handler_contacts();
    assert_eq!(prepared_contacts, (before.0 + 1, before.1 + 1));
    let original_world = head(application);
    application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            application
                .runtime()
                .fail_next_durable_append_for_test(&execution);
        })
        .unwrap();
    let WorthQuerySourceOnlyAdvance::Progress(operation) = prepared.publish().unwrap() else {
        panic!("real failed durable settlement retains the original operation");
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(head(application), original_world);
    assert_eq!(
        host::read(application),
        [Row {
            primary: 23,
            secondary: 7
        }]
    );
    {
        let page = application
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let recovery = application
            .runtime()
            .readmit_product_publication_recovery(page.rows()[0].handle())
            .unwrap();
        let partial = recovery.inspect().unwrap();
        assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
        assert!(partial.relational_requires_settlement());
        assert_eq!(partial.owner_effect_count(), 1);
    }
    let (operation, denial) = operation
        .close(&request)
        .err()
        .expect("partial native effects cannot close");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    assert_eq!(handler_contacts(), prepared_contacts);

    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(application.installed_schema(), &fresh_scope);
    let fresh = application
        .runtime()
        .request(&fresh_principal, &fresh_scope);
    let operation = match operation.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("original combined recovery refused: {denial:?}")
        }
        _ => panic!("the original owner enters recovered publication"),
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RecoveryPerformed
    );
    let completed = match operation.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("original receipt resolution refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Progress(operation) => {
            panic!("original receipt remains in {:?}", operation.posture())
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!(
                "original receipt resolves terminal: {:?}",
                terminal.native_keyed_resolution()
            )
        }
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: None,
        publication: Some(_),
        ..
    } = completed.evidence()
    else {
        panic!("recovered source completion retains actual publication evidence");
    };
    composite::assert_combined(receipt);
    assert_eq!(
        receipt.aftermath_causality().unwrap().recorded_source(),
        Some(&source.correction_source())
    );
    assert_ne!(
        receipt.committed_product_publication().signal_publication(),
        source.committed_product_publication().signal_publication()
    );
    assert_eq!(
        host::read(application),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    assert_eq!(
        handler_contacts(),
        prepared_contacts,
        "recovery consumes retained native work"
    );
    let recovered_world = head(application);
    let (_, WorthQuerySourceCompletion::Recovered { receipt, .. }) = completed
        .close_with_evidence(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("original completed owner closes: {denial:?}"))
    else {
        panic!("close transfers the original recovered evidence");
    };

    let WorthQueryOperationCorrectionPreparation::Continued(
        WorthQuerySourceOnlyAdvance::Completed(replayed),
    ) = fresh
        .corrections()
        .reverse(&selected)
        .idempotency("combined-recovery")
        .limits(limits())
        .prepare_operation::<CorrectDefinition, Program, Action>(application)
        .unwrap()
    else {
        panic!("same command returns its original performed receipt");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt: replay, ..
    } = replayed.evidence()
    else {
        panic!("replay preserves recovered evidence provenance");
    };
    assert_eq!(replay.commit_reference(), receipt.commit_reference());
    assert_eq!(
        replay.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        replay.committed_product_publication().signal_publication(),
        receipt.committed_product_publication().signal_publication()
    );
    assert_eq!(
        replay
            .committed_product_publication()
            .conditional_definition_generation(),
        receipt
            .committed_product_publication()
            .conditional_definition_generation()
    );
    assert_eq!(
        replay.aftermath_causality().unwrap().recorded_source(),
        receipt.aftermath_causality().unwrap().recorded_source()
    );
    assert_eq!(head(application), recovered_world);
    assert_eq!(handler_contacts(), prepared_contacts);
    replayed
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("replayed owner closes: {denial:?}"));
    assert_eq!(
        composite::observe(application, WorthQueryConditionalSignalDecision::Eligible),
        1
    );
    assert_eq!(host::read(application)[0].secondary, 99);
    assert!(application
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
}
