//! Run with `cargo run -p worth-query-certification --example ordinary_product_workflow`.

pub mod product_workflow_support;

use product_workflow_support::schema::AmendTemporalInput;
use product_workflow_support::{
    principal, AmendTemporalIntent, ExampleApplication, TemporalIntentRead,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
};

fn main() {
    std::thread::Builder::new()
        .name("ordinary-product-workflow".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("the bounded example thread must start")
        .join()
        .expect("the ordinary product workflow must not unwind");
}

fn run() {
    let mut application = ExampleApplication::publish("blocked");
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&application, &scope);
    let request = application.runtime.request(&principal, &scope);
    let retained = application
        .runtime
        .with_host_advancement(|host_phase| {
            request
                .retain_read(&host_phase)
                .expect("the initial application read must be retainable")
        })
        .expect("installed host call opens");
    let initial = request
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the application entry must read the initial row");
    assert_eq!(initial.rows()[0].input, "payload");

    // compiled-example: begin
    let operation = request
        .mutate(AmendTemporalIntent {
            identity: "intent-1".to_owned(),
            amendment: AmendTemporalInput {
                revision: 2,
                due: 11,
                lifecycle: "active".to_owned(),
                input: "published-through-application-entry".to_owned(),
                gate: "ready".to_owned(),
            },
        })
        .without_source()
        .idempotency_owned(0x51_u64).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<product_workflow_support::program::TemporalExampleProgram,
            product_workflow_support::program::AmendTemporalAction>(&application.runtime)
        .unwrap_or_else(|(_, denial)| panic!("declared temporal action: {denial:?}"));
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("fresh attempt refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("this successful native attempt has no pending publication")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("native terminal: {:?}", terminal.native_outcome())
        }
    };
    let committed_version = match completed.evidence() {
        WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } => {
            assert_eq!(result.revision, 2);
            receipt.commit_reference().version_id.as_u64()
        }
        WorthQuerySourceCompletion::Recovered { .. } => {
            panic!("a new key must produce the actual handler result")
        }
    };
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("native cleanup refused: {denial:?}"));
    // compiled-example: end

    let current = application
        .runtime
        .request(&principal, &scope)
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the first successor must be readable through the same entry");
    assert_eq!(
        current.rows()[0].input,
        "published-through-application-entry"
    );
    assert_eq!(
        current.receipt().inspect().basis().version(),
        committed_version
    );

    let original = application
        .runtime
        .request(&principal, &scope)
        .at(&retained)
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the retained application request must preserve the original basis");
    assert_eq!(original.rows()[0].input, "payload");

    application
        .runtime
        .close_conditional_runtime()
        .expect("the application runtime must cleanly close");
    println!("read, mutated, retained, and cleaned up through the application entry");
}
