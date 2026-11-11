//! Codec/installed-meaning kernels. These claim no Native/World publication.
use super::*;
mod installation;
use installation::{installed, OutboxContractProfile};
use worth_foundational::facade::CanonicalDigestId;

fn fields(source: &dyn InstalledOperationSource) -> WorthQueryDispatchOutboxRestoredFields {
    let view = source.installed_view();
    let InstalledExternalEffectContract::Declared {
        correlation_family,
        effect,
        protocol,
        maximum_payload_bytes,
        ..
    } = view.contracts().external_effect()
    else {
        panic!("fixture declares an external effect")
    };
    WorthQueryDispatchOutboxRestoredFields {
        correlation: ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            [7; 32],
        )),
        correlation_family: correlation_family.clone(),
        effect: effect.clone(),
        protocol_identity: protocol.identity().clone(),
        protocol_version: protocol.version(),
        maximum_payload_bytes: *maximum_payload_bytes,
        payload: vec![1, 2],
        outcome_identity: 9,
        operation_slot: Some(view.operation().to_owned()),
    }
}

#[test]
fn original_definition_rebinds_across_independent_real_installations() {
    let original = installed(OutboxContractProfile::Inbound);
    let current = installed(OutboxContractProfile::Inbound);
    let decoded = fields(original.operation.as_ref());
    let rebound = WorthQueryRecoveredDispatchOutbox::from_source_row(
        data_record(),
        decoded,
        &original.intent,
        &current.catalog,
    );
    assert_eq!(
        rebound.inbound().unwrap().unwrap().source_identity(),
        "source-a"
    );
    assert_eq!(rebound.payload(), [1, 2]);
}

#[test]
fn equal_outbound_fields_cannot_rebind_changed_inbound_source_or_limits() {
    let original = installed(OutboxContractProfile::Inbound);
    let decoded = fields(original.operation.as_ref());
    for changed in [
        installed(OutboxContractProfile::ChangedInboundSource),
        installed(OutboxContractProfile::ChangedInboundLimits),
    ] {
        assert_eq!(
            decoded,
            fields(changed.operation.as_ref()),
            "outbound fields alone do not bind inbound meaning"
        );
        assert_ne!(
            original
                .operation
                .installed_view()
                .definition_identity_bytes(),
            changed
                .operation
                .installed_view()
                .definition_identity_bytes()
        );
        let rebound = WorthQueryRecoveredDispatchOutbox::from_source_row(
            data_record(),
            decoded.clone(),
            &original.intent,
            &changed.catalog,
        );
        assert_eq!(rebound.inbound(), Err(Denial::RecordMismatch));
        assert_eq!(rebound.record_ref(), &data_record());
        assert_eq!(rebound.payload(), [1, 2]);
        assert_eq!(rebound.effect(), decoded.effect);
        assert_eq!(
            rebound.correlation_family(),
            decoded.correlation_family.as_str()
        );
        assert_eq!(rebound.outcome_identity(), 9);
        assert_eq!(rebound.operation_slot(), decoded.operation_slot.as_deref());
    }
}

#[test]
fn missing_or_unverifiable_binding_never_claims_no_inbound() {
    let original = installed(OutboxContractProfile::Inbound);
    let mut decoded = fields(original.operation.as_ref());
    assert!(matches!(
        readmit_contract(&decoded, "malformed", &original.catalog),
        Err(Denial::Malformed)
    ));
    let absent = original.intent.replace(
        &format!(
            "operation={}",
            original
                .operation
                .installed_view()
                .definition_identity_bytes()
                .map(|b| format!("{b:02x}"))
                .concat()
        ),
        "operation=-",
    );
    assert!(matches!(
        readmit_contract(&decoded, &absent, &original.catalog),
        Err(Denial::NotAuthoritative)
    ));
    assert!(matches!(
        readmit_contract(
            &decoded,
            &original.intent,
            &InstalledOperationCatalog::default()
        ),
        Err(Denial::Missing)
    ));
    decoded.operation_slot = None;
    assert!(matches!(
        readmit_contract(&decoded, &original.intent, &original.catalog),
        Err(Denial::NotAuthoritative)
    ));
}

#[test]
fn only_a_matching_installed_absence_is_no_inbound() {
    let original = installed(OutboxContractProfile::NoInbound);
    let current = installed(OutboxContractProfile::NoInbound);
    let rebound = WorthQueryRecoveredDispatchOutbox::from_source_row(
        data_record(),
        fields(original.operation.as_ref()),
        &original.intent,
        &current.catalog,
    );
    assert_eq!(rebound.inbound(), Ok(None));
    assert_eq!(rebound.payload(), [1, 2]);
}

// Descriptive identifiers for the DATA constructor; never a Native publication.
fn data_record() -> RecordRef {
    use worth_relational::facade::identity::{EntityId, PartitionId};
    RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1))
}
