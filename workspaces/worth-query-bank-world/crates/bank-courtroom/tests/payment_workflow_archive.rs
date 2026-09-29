//! The real installed Bank payment vocabulary readmits its portable wait.

use bank_domain::model::BankPrincipalId;
use bank_domain::schema::{
    approved_payment_inbound_binding, ApprovedBusinessPaymentWorkflow,
    ApprovedPaymentSettlementEffect, ApprovedPaymentSettlementRequestBinding, BankSchema,
};
use bank_server::{approved_business_payment_definition, BankIdentityRuntime, BankPrincipalSeed};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAwaitInbound,
    ApplicationWorkflowCommandAdapter, ApplicationWorkflowInboundRef, ApplicationWorkflowNodeKind,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind;
use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, encode_workflow_definition_draft,
    WorthQueryPackageArchiveLimits, WorthQueryWorkflowDefinitionDraftDenialKind,
};

#[test]
fn approved_payment_wait_round_trips_through_v3_and_binds_to_live_bank_vocabulary() {
    let runtime = installed_bank();
    let installed = runtime.approved_payment_workflow_spec();
    let original = approved_business_payment_definition().expect("Bank authors the payment wait");
    let bytes =
        encode_workflow_definition_draft(&original, WorthQueryPackageArchiveLimits::DEFAULT)
            .expect("the payment definition fits the archive budget");
    assert_eq!(&bytes[..6], b"WQWD\0\x03", "the authored wait writes v3");

    let rebuilt = decode_workflow_definition_draft(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the bounded v3 draft decodes")
        .author(installed)
        .expect("every portable member resolves against the installed Bank vocabulary")
        .validate()
        .expect("the resolved payment graph validates");
    assert_eq!(rebuilt.content_identity(), original.content_identity());
    assert_eq!(
        encode_workflow_definition_draft(&rebuilt, WorthQueryPackageArchiveLimits::DEFAULT)
            .expect("the rebuilt definition encodes"),
        bytes
    );
    let bound = installed
        .bind_definition(rebuilt)
        .expect("the rebuilt typed wait binds to the installed Bank schema");
    assert_eq!(
        bound.definition().content_identity(),
        original.content_identity()
    );
    assert!(bound.definition().nodes().iter().any(|node| {
        matches!(node.kind(), ApplicationWorkflowNodeKind::AwaitInbound(awaited)
            if node.identity().as_str() == "await-inbound"
                && awaited.origin().as_str() == "apply"
                && awaited.inbound().source_identity() == "rail-primary")
    }));
}

#[test]
fn changed_or_missing_payment_wait_members_cannot_reauthor() {
    let runtime = installed_bank();
    let definition = approved_business_payment_definition().expect("Bank authors the payment wait");
    let bytes =
        encode_workflow_definition_draft(&definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .expect("the payment definition fits the archive budget");
    for (from, to, expected) in [
        (
            "rail-primary",
            "rail-uninstalled",
            WorthQueryWorkflowDefinitionDraftDenialKind::ChangedMember,
        ),
        (
            "ApprovePaymentOperation",
            "MissingPaymentOperation",
            WorthQueryWorkflowDefinitionDraftDenialKind::UnknownMember,
        ),
    ] {
        let altered = replace_framed(&bytes, from, to);
        let authored =
            decode_workflow_definition_draft(&altered, WorthQueryPackageArchiveLimits::DEFAULT)
                .expect("the altered draft remains structurally valid")
                .author(runtime.approved_payment_workflow_spec());
        let denial = match authored {
            Ok(_) => panic!("portable text cannot substitute an installed Bank member"),
            Err(denial) => denial,
        };
        assert_eq!(denial.kind(), expected);
    }
}

#[derive(Clone)]
struct ForeignPaymentSettlementEffect;

impl ApplicationEffectMarkerIdentity<BankSchema> for ForeignPaymentSettlementEffect {
    type PayloadBinding = ApprovedPaymentSettlementRequestBinding;
    const IDENTIFIER: &'static str =
        <ApprovedPaymentSettlementEffect as ApplicationEffectMarkerIdentity<BankSchema>>::IDENTIFIER;
}

#[test]
fn same_name_foreign_effect_marker_cannot_bind_even_with_identical_archive_bytes() {
    let runtime = installed_bank();
    let original = approved_business_payment_definition().expect("Bank authors the payment wait");
    let declared = approved_payment_inbound_binding();
    let foreign_binding =
        ApplicationInboundOccurrenceBinding::<ForeignPaymentSettlementEffect>::new(
            declared.protocol().clone(),
            declared.source_identity(),
            declared.limits(),
        )
        .expect("the foreign marker has the same portable inbound fields");
    let mut commands = original
        .nodes()
        .iter()
        .map(|node| {
            let kind = match node.kind() {
                ApplicationWorkflowNodeKind::AwaitInbound(awaited) => {
                    ApplicationWorkflowNodeKind::AwaitInbound(ApplicationWorkflowAwaitInbound::new(
                        awaited.origin().clone(),
                        ApplicationWorkflowInboundRef::declared::<
                            ApprovedBusinessPaymentWorkflow,
                            ForeignPaymentSettlementEffect,
                        >(foreign_binding.clone()),
                        awaited.wait(),
                    ))
                }
                other => other.clone(),
            };
            ApplicationWorkflowAuthoringCommand::Node(node.identity().clone(), kind)
        })
        .collect::<Vec<_>>();
    commands.push(ApplicationWorkflowAuthoringCommand::Start(
        original.start().clone(),
    ));
    commands.extend(
        original
            .connections()
            .iter()
            .cloned()
            .map(ApplicationWorkflowAuthoringCommand::Connection),
    );
    let foreign = ApplicationWorkflowCommandAdapter::author::<ApprovedBusinessPaymentWorkflow>(
        original.identity().as_str(),
        original.limits(),
        commands,
    )
    .expect("portable same-name marker authors")
    .validate()
    .expect("the graph remains valid");
    let encode = |definition| {
        encode_workflow_definition_draft(definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .expect("the definition fits the archive budget")
    };
    assert_eq!(encode(&foreign), encode(&original));
    let denial = match runtime
        .approved_payment_workflow_spec()
        .bind_definition(foreign)
    {
        Ok(_) => panic!("same-name Rust effect marker cannot claim the installed wait"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::UnsupportedDefinitionMember
    );
}

fn installed_bank() -> BankIdentityRuntime {
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

fn replace_framed(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
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
