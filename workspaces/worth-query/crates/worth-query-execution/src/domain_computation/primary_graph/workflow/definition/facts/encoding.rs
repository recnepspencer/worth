use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationWorkflowConnectionKind;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::super::codec::WorkflowConnectionTag;
use super::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
    WorthQueryWorkflowLayout,
};

pub(super) fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

pub(super) fn create_connection(
    layout: &WorthQueryWorkflowLayout,
    reference: &CreatedEntityRef,
    connection: ApplicationWorkflowConnectionKind,
    creation_partition: WorthQueryApplicationCreationPartition,
) -> WorthQueryApplicationRealizedEffect {
    let retry = match &connection {
        ApplicationWorkflowConnectionKind::Retry(retry) => Some(retry),
        _ => None,
    };
    let tag = WorkflowConnectionTag::from_declared(connection.clone());
    let mut fields = BTreeMap::from([
        (
            layout.connection.family.clone(),
            AspectValue::UInt64(tag.family()),
        ),
        (
            layout.connection.variant.clone(),
            AspectValue::UInt64(tag.variant()),
        ),
    ]);
    if let Some(retry) = retry {
        fields.insert(layout.connection.retry_reason.clone(), text(retry.reason()));
        fields.insert(
            layout.connection.retry_maximum_attempts.clone(),
            AspectValue::UInt64(u64::from(retry.maximum_attempts())),
        );
    }
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: reference.kind_id,
        key: raw_key(reference),
        fields,
        partition: creation_partition,
    }
}

pub(super) fn create_relation(
    kind_id: KindId,
    key: impl Into<String>,
    source: EntityReference,
    target: EntityReference,
) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: kind_id,
        key: key.into(),
        from: source,
        to: target,
    }
}

pub(super) fn created(
    partition_id: PartitionId,
    kind_id: KindId,
    key: impl Into<String>,
) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id,
        kind_id,
        client_key: ClientKey::raw(key),
    }
}

pub(super) fn raw_key(reference: &CreatedEntityRef) -> String {
    reference
        .client_key
        .as_raw_str()
        .expect("workflow publication uses raw owner-issued client keys")
        .to_owned()
}

pub(super) fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
