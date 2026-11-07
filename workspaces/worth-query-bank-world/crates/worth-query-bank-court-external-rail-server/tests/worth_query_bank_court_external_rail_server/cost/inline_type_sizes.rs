//! Layout guards for the genuine Bank workflow, without constructing its owners.
use bank_domain::schema::{ApprovedBusinessPaymentApplyIntent, BankSchema};
use bank_server::{ApprovedPaymentEffectAction, BankApplication};
use worth_query_host::facade::application_entry::*;

type Operation = WorthQueryWorkflowOperation<
    BankSchema,
    ApprovedBusinessPaymentApplyIntent,
    BankApplication,
    ApprovedPaymentEffectAction,
>;
type Completed = WorthQueryCompletedWorkflowOperation<
    BankSchema,
    ApprovedBusinessPaymentApplyIntent,
    BankApplication,
    ApprovedPaymentEffectAction,
>;
type Terminal = WorthQueryTerminalWorkflowOperation<
    BankSchema,
    ApprovedBusinessPaymentApplyIntent,
    BankApplication,
    ApprovedPaymentEffectAction,
>;

#[test]
fn bank_workflow_carriers_stay_within_declared_inline_ceilings() {
    let rows = [
        row::<Operation>(),
        row::<Completed>(),
        row::<Terminal>(),
        row::<WorthQueryWorkflowOperationAdvance<Operation, Completed, Terminal>>(),
        row::<WorthQueryWorkflowAcceptance<Completed>>(),
        row::<
            WorthQueryHistoryEditOutcome<
                BankSchema,
                ApprovedBusinessPaymentApplyIntent,
                BankApplication,
                ApprovedPaymentEffectAction,
            >,
        >(),
    ];
    for (name, size) in &rows {
        println!("APPLICATION_TYPE_SIZE|{name}|{size}|{}", 128 * 1024);
    }
    let exceeded: Vec<_> = rows.iter().filter(|(_, size)| *size > 128 * 1024).collect();
    assert!(
        exceeded.is_empty(),
        "inline carrier ceiling exceeded: {exceeded:?}"
    );
}

fn row<T>() -> (&'static str, usize) {
    (std::any::type_name::<T>(), std::mem::size_of::<T>())
}
