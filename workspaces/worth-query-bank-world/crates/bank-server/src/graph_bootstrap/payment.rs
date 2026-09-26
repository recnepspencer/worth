use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};

use super::{
    account_key, approval_key, business_key, entity_key, payment_key, payment_workflow_grant_key,
    principal_key,
};

pub(super) fn bind_payments(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for payment in snapshot.payments() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                PaymentIntent::reference(),
                entity_key(payment_key(payment.id())),
            )
            .field(PaymentIdentityField::reference(), payment.id())
            .field(PaymentAmount::reference(), payment.amount())
            .field(PaymentStatusField::reference(), payment.status()),
        )?;
        if payment.deciding_principal().is_some() {
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(
                Approval::reference(),
                entity_key(approval_key(payment.id())),
            ))?;
        }
        for grantee in snapshot.payment_approval_grantees(payment) {
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    ApprovedBusinessPaymentGrant::reference(),
                    entity_key(payment_workflow_grant_key(payment.id(), grantee)),
                )
                .field(
                    ApprovedBusinessPaymentGrantAction::reference(),
                    "manage-approved-business-payment-workflow".to_owned(),
                )
                .field(
                    ApprovedBusinessPaymentGrantPurpose::reference(),
                    "business-payment-approval".to_owned(),
                )
                .field(
                    ApprovedBusinessPaymentGrantStatus::reference(),
                    "active".to_owned(),
                )
                .field(
                    ApprovedBusinessPaymentGrantWorkflow::reference(),
                    payment.id(),
                )
                .field(ApprovedBusinessPaymentGrantNotBefore::reference(), 0)
                .field(ApprovedBusinessPaymentGrantNotAfter::reference(), u64::MAX)
                .field(ApprovedBusinessPaymentGrantDelegationLimit::reference(), 0),
            )?;
        }
    }
    Ok(())
}

pub(super) fn bind_payment_relations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for payment in snapshot.payments() {
        let payment_key_value = payment_key(payment.id());
        bind_payment_topology(graph, payment, &payment_key_value)?;
        for grantee in snapshot.payment_approval_grantees(payment) {
            let grant = payment_workflow_grant_key(payment.id(), grantee);
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                ApprovedBusinessPaymentGrantResource::reference(),
                format!("approved-payment-grant-resource:{grant}"),
                entity_key(grant.clone()),
                entity_key(payment_key_value.clone()),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                ApprovedBusinessPaymentGrantGrantor::reference(),
                format!("approved-payment-grant-grantor:{grant}"),
                entity_key(principal_key(payment.initiator().get())),
                entity_key(grant.clone()),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                ApprovedBusinessPaymentGrantGrantee::reference(),
                format!("approved-payment-grant-grantee:{grant}"),
                entity_key(principal_key(grantee.get())),
                entity_key(grant),
            ))?;
        }
        if let Some(decider) = payment.deciding_principal() {
            let approval_key_value = approval_key(payment.id());
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                PaymentApproval::reference(),
                format!("payment-approval:{}", payment.id().canonical_text()),
                entity_key(payment_key_value.clone()),
                entity_key(approval_key_value.clone()),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                ApprovalPrincipal::reference(),
                format!("approval-principal:{}", payment.id().canonical_text()),
                entity_key(approval_key_value),
                entity_key(principal_key(decider.get())),
            ))?;
        }
    }
    Ok(())
}

fn bind_payment_topology(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    payment: &bank_domain::payments::BusinessPayment,
    payment_key_value: &str,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        PaymentSource::reference(),
        format!("payment-source:{}", payment.id().canonical_text()),
        entity_key(payment_key_value.to_owned()),
        entity_key(account_key(payment.source())),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        PaymentDestination::reference(),
        format!("payment-destination:{}", payment.id().canonical_text()),
        entity_key(payment_key_value.to_owned()),
        entity_key(account_key(payment.destination())),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        PaymentBusiness::reference(),
        format!("payment-business:{}", payment.id().canonical_text()),
        entity_key(payment_key_value.to_owned()),
        entity_key(business_key(payment.business().get())),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        PaymentInitiator::reference(),
        format!("payment-initiator:{}", payment.id().canonical_text()),
        entity_key(principal_key(payment.initiator().get())),
        entity_key(payment_key_value.to_owned()),
    ))
}
