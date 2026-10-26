//! Aggregate answer meaning; source, execution custody and incident history are excluded.
use super::*;
use crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation;

pub(super) fn append_values(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    body: &AggregateObservation,
) {
    match append(entries, prefix, body, &mut || {
        Ok::<(), std::convert::Infallible>(())
    }) {
        Ok(()) => (),
        Err(never) => match never {},
    }
}
pub(super) fn append<E>(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    body: &AggregateObservation,
    poll: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    poll()?;
    kind(entries, prefix, "complete-native-aggregate");
    let key = body.key();
    number_u64(
        entries,
        prefix,
        "entity-kind",
        key.entity_kind().as_u32().into(),
    );
    number_u64(
        entries,
        prefix,
        "relation-kind",
        key.relation_kind().as_u32().into(),
    );
    push(
        entries,
        format!("{prefix}.relation"),
        CanonicalBasisEntryKind::Identity,
        text(key.relation()),
    );
    entity(entries, prefix, "selector", key.member());
    push(
        entries,
        format!("{prefix}.definition"),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(
            *body.native().definition().bytes(),
        )),
    );
    push(
        entries,
        format!("{prefix}.identity"),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*body.native().identity().bytes())),
    );
    number(entries, prefix, "member-count", body.native().len());
    for (index, member) in body.native().members().enumerate() {
        poll()?;
        entity(entries, prefix, &format!("member.{index}"), member);
    }
    poll()?;
    Ok(())
}
pub(in super::super) fn entries_checked<E>(
    index: usize,
    body: &AggregateObservation,
    mut poll: impl FnMut() -> Result<(), E>,
) -> Result<Vec<CanonicalBasisEntry>, E> {
    poll()?;
    let mut entries = Vec::new();
    let prefix = format!("fact.{index}");
    let mut locator = String::new();
    body.key()
        .write_locator(&mut locator)
        .expect("writing an aggregate locator to String");
    push(
        &mut entries,
        format!("{prefix}.locator"),
        CanonicalBasisEntryKind::Locator,
        text(locator),
    );
    append(&mut entries, &prefix, body, &mut poll)?;
    Ok(entries)
}
