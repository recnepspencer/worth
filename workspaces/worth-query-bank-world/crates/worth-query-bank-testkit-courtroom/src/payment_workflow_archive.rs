//! The real installed Bank payment vocabulary readmits its portable wait.

pub use bank_domain::model::BankPrincipalId;
pub use bank_domain::schema::{
    approved_payment_inbound_binding, ApprovedBusinessPaymentWorkflow,
    ApprovedPaymentSettlementEffect, ApprovedPaymentSettlementRequestBinding, BankSchema,
};
pub use bank_server::{
    approved_business_payment_definition, BankIdentityRuntime, BankPrincipalSeed,
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAwaitInbound,
    ApplicationWorkflowCommandAdapter, ApplicationWorkflowInboundRef, ApplicationWorkflowNodeKind,
};
pub use worth_query_host::facade::declaration::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
};
pub use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
pub use worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind;
pub use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, encode_workflow_definition_draft,
    WorthQueryPackageArchiveLimits, WorthQueryWorkflowDefinitionDraftDenialKind,
};

#[derive(Clone)]
pub struct ForeignPaymentSettlementEffect;

impl ApplicationEffectMarkerIdentity<BankSchema> for ForeignPaymentSettlementEffect {
    type PayloadBinding = ApprovedPaymentSettlementRequestBinding;
    const IDENTIFIER: &'static str =
        <ApprovedPaymentSettlementEffect as ApplicationEffectMarkerIdentity<BankSchema>>::IDENTIFIER;
}

pub fn installed_bank() -> BankIdentityRuntime {
    BankIdentityRuntime::install([BankPrincipalSeed::enabled(
        BankPrincipalId::new(1).unwrap(),
        WorthQueryExternalPrincipalIdentity::new(
            "https://payment-archive.bank.test.invalid",
            "archive-principal",
        )
        .unwrap(),
    )])
    .expect("Bank installs its payment schema and workflow vocabulary")
}

pub fn replace_framed(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    fn frame(text: &str) -> Vec<u8> {
        let mut frame = u32::try_from(text.len()).unwrap().to_be_bytes().to_vec();
        frame.extend_from_slice(text.as_bytes());
        frame
    }
    let from = frame(from);
    let to = frame(to);
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut replacements = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(&from) {
            output.extend_from_slice(&to);
            index += from.len();
            replacements += 1;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    assert!(replacements > 0, "the authored draft carries the marker");
    output
}
