//! Old-versus-new evidence for Bank's macro-declared workflow controls.
//!
//! `control.rs` moved from a hand-written local macro to
//! `worth_query_workflow_controls!`. The schema fingerprint below was taken
//! from the hand-written declaration on the same branch, before the move, and
//! the literals are the hand-written identities, so a drift in either the
//! installed schema or any wire identity fails here.
use bank_domain::schema::BankSchema;

/// FNV-1a over the declaration's members and canonical schema identity.
pub fn bank_schema_fingerprint() -> u64 {
    let Ok(declaration) =
        worth_query_decl::facade::application_schema::fragment::declare(BankSchema)
    else {
        return 0;
    };
    let rendered = format!(
        "{:?}\n{:?}",
        declaration.erased().members(),
        declaration.identity()
    );
    rendered
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

/// The fingerprint of the hand-written control declaration, captured before
/// the migration.
pub const HAND_WRITTEN_BANK_SCHEMA_FINGERPRINT: u64 = 10_599_574_432_653_203_567;

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
