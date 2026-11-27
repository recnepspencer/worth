//! The hand-written wire identities of Bank's workflow controls.
//!
//! `control.rs` moved from a hand-written local macro to
//! `worth_query_workflow_controls!`; at the move (commit `f9ada622c0`) the
//! whole installed Bank schema was compared byte for byte with the
//! hand-written declaration. These literals keep pinning every control's wire
//! identity without tying the court to unrelated Bank schema edits.

/// The four controls' hand-written wire identities: capability marker,
/// capability portable type, binding, handler and command.
pub const HAND_WRITTEN_CONTROL_IDENTITIES: [[&str; 5]; 4] = [
    [
        "ApprovedBusinessPaymentApproval",
        "worth.bank.approved-business-payment-approval.v1",
        "worth.bank.approved-business-payment-approval.binding.v1",
        "worth.bank.approved-business-payment-approval.handler.v1",
        "worth.bank.approved-business-payment-approval.command.v1",
    ],
    [
        "ApprovedBusinessPaymentAuthoring",
        "worth.bank.approved-business-payment-authoring.v1",
        "worth.bank.approved-business-payment-authoring.binding.v1",
        "worth.bank.approved-business-payment-authoring.handler.v1",
        "worth.bank.approved-business-payment-authoring.command.v1",
    ],
    [
        "ApprovedBusinessPaymentInstanceStart",
        "worth.bank.approved-business-payment-instance-start.v1",
        "worth.bank.approved-business-payment-instance-start.binding.v1",
        "worth.bank.approved-business-payment-instance-start.handler.v1",
        "worth.bank.approved-business-payment-instance-start.command.v1",
    ],
    [
        "ApprovedBusinessPaymentAdvance",
        "worth.bank.approved-business-payment-advance.v1",
        "worth.bank.approved-business-payment-advance.binding.v1",
        "worth.bank.approved-business-payment-advance.handler.v1",
        "worth.bank.approved-business-payment-advance.command.v1",
    ],
];
