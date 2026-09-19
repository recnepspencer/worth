use std::collections::BTreeSet;

use super::*;

pub(super) fn assert_portable_contract_spine(package: &WorthQueryValidatedPortableDomainPackage) {
    let spine = package.application_contract_spine();
    assert_eq!(spine.native_aspects().len(), 2);
    let state = native_aspect(spine, "AccountState");
    assert_eq!(state.contract().identity(), AspectIdentity(0x9161_1051));
    assert_eq!(state.contract().revision(), AspectContractRevision(2));
    assert_eq!(
        state
            .fields()
            .map(FieldKey::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["AccountBalance", "AccountLimit", "AccountStatus"])
    );
    let audit = native_aspect(spine, "AccountAudit");
    assert_eq!(audit.contract().identity(), AspectIdentity(0x9161_1052));
    assert_eq!(
        audit.fields().map(FieldKey::as_str).collect::<Vec<_>>(),
        ["AuditSequence"]
    );

    let update = spine
        .operations()
        .iter()
        .find(|record| record.operation() == "UpdateAccount")
        .expect("the mutation contract is retained at validation");
    assert_eq!(update.graph_reads().len(), 3);
    assert_read_family(update, state.contract());
    assert_touch_family(update, state.contract());
    assert_external_contract(spine);
}

fn native_aspect<'a>(
    spine: &'a worth_query_host::facade::domain::WorthQueryPortableApplicationContractSpine,
    aspect: &str,
) -> &'a worth_query_host::facade::domain::WorthQueryPortableNativeAspectContractRecord {
    spine
        .native_aspects()
        .iter()
        .find(|record| record.aspect().as_str() == aspect)
        .expect("every declared native aspect is retained")
}

fn assert_read_family(
    update: &worth_query_host::facade::domain::WorthQueryPortableApplicationOperationContractRecord,
    state: &worth_foundational::facade::AspectContract,
) {
    assert!(update.graph_reads().iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationGraphReadScope::Entity { entity, .. } if entity == "Account"
    )));
    assert!(update.graph_reads().iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationGraphReadScope::NativeProjection { contract, mask, .. }
            if contract == state
                && mask.paths() == ["AccountLimit", "AccountStatus"]
                    .map(|field| CanonicalFieldPath::single(FieldKey::new(field).unwrap()))
    )));
    assert!(update.graph_reads().iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationGraphReadScope::Relation { relation, from, to, .. }
            if relation == "ObservedAccount" && from == "Account" && to == "Account"
    )));
}

fn assert_touch_family(
    update: &worth_query_host::facade::domain::WorthQueryPortableApplicationOperationContractRecord,
    state: &worth_foundational::facade::AspectContract,
) {
    let touches = update.touches();
    assert_eq!(touches.len(), 5);
    assert!(touches.iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationTouchScope::CreateEntity { entity, .. } if entity == "Account"
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationTouchScope::DeleteEntity { entity, .. } if entity == "Account"
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationTouchScope::WriteField { contract, field_path, .. }
            if contract == state
                && field_path == &CanonicalFieldPath::single(FieldKey::new("AccountBalance").unwrap())
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationTouchScope::LinkRelation { relation, .. }
            if relation == "ChangedAccount"
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        WorthQueryPortableOperationTouchScope::UnlinkRelation { relation, .. }
            if relation == "ChangedAccount"
    )));
}

fn assert_external_contract(
    spine: &worth_query_host::facade::domain::WorthQueryPortableApplicationContractSpine,
) {
    let notice = spine
        .operations()
        .iter()
        .find(|record| record.operation() == "EmitAccountNotice")
        .expect("the external operation contract is retained at validation");
    assert_eq!(notice.emissions(), &["AccountNoticeEffect"]);
    let external = notice
        .external_effect()
        .expect("external contract is retained");
    assert_eq!(external.correlation_family().as_str(), CORRELATION_FAMILY);
    assert_eq!(
        external.payload_type().as_str(),
        "worth.query.test.host.account_notice.v1"
    );
    assert_eq!(external.maximum_payload_bytes(), 8);
    assert_eq!(
        external.protocol(),
        &<AccountNoticeBinding as ApplicationExternalEffectBinding>::PROTOCOL
    );
    assert_eq!(
        notice.reconciliation().unwrap().procedure_slot(),
        RECONCILIATION_SLOT
    );
}
