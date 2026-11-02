//! Complete original derived truth; live issuer/payment do not become portable grants.
use super::*;
use crate::domain_computation::primary_graph::derived_source::DerivedSourceObservation;
pub(super) fn append_values(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    body: &DerivedSourceObservation,
) {
    append(entries, prefix, body, &mut || {
        Ok::<_, std::convert::Infallible>(())
    })
    .unwrap();
}
fn append<E>(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    body: &DerivedSourceObservation,
    poll: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    poll()?;
    kind(entries, prefix, "complete-live-derived-source");
    let key = body.key();
    for (label, value) in [
        ("relation", key.relation()),
        ("schema", key.identity().schema.0.as_str()),
        ("definition", key.identity().name.as_str()),
    ] {
        for _ in value.bytes() {
            poll()?;
        }
        push(
            entries,
            format!("{prefix}.{label}"),
            CanonicalBasisEntryKind::Identity,
            text(value),
        );
    }
    number_u64(
        entries,
        prefix,
        "schema-version",
        u64::from(key.identity().version.0),
    );
    entity(entries, prefix, "root", key.root());
    push(
        entries,
        format!("{prefix}.direction"),
        CanonicalBasisEntryKind::Value,
        text(
            if key.direction() == worth_foundational::facade::RelationDirection::Reverse {
                "reverse"
            } else {
                "forward"
            },
        ),
    );
    let data = body.native().data();
    number(entries, prefix, "endpoint-count", data.endpoints().len());
    for (i, member) in data.endpoints().iter().enumerate() {
        poll()?;
        entity(
            entries,
            prefix,
            &format!("endpoint.{i}"),
            member.entity().expect("physical root-path entrance"),
        );
    }
    number(
        entries,
        prefix,
        "entity-count",
        data.entity_dependencies().len(),
    );
    for (i, (id, kind)) in data.entity_dependencies().iter().enumerate() {
        poll()?;
        let at = format!("{prefix}.entity.{i}");
        entity(entries, &at, "id", *id);
        number_u64(entries, &at, "kind", kind.as_u32() as u64);
    }
    number(
        entries,
        prefix,
        "relation-count",
        data.physical_relations().len(),
    );
    for (i, edge) in data.physical_relations().iter().enumerate() {
        poll()?;
        let at = format!("{prefix}.relation.{i}");
        record(entries, &at, "id", edge.relation_id);
        entity(entries, &at, "source", edge.source);
        entity(entries, &at, "target", edge.target);
        number_u64(entries, &at, "kind", edge.kind_id.as_u32() as u64);
    }
    number(
        entries,
        prefix,
        "adjacency-count",
        data.adjacency_dependencies().len(),
    );
    for (i, probe) in data.adjacency_dependencies().iter().enumerate() {
        poll()?;
        let at = format!("{prefix}.adjacency.{i}");
        entity(entries, &at, "anchor", probe.anchor);
        number_u64(entries, &at, "kind", probe.relation.as_u32() as u64);
        push(
            entries,
            format!("{at}.direction"),
            CanonicalBasisEntryKind::Value,
            text(if probe.outgoing {
                "outgoing"
            } else {
                "incoming"
            }),
        );
    }
    poll()?;
    Ok(())
}
pub(in super::super) fn entries_checked<E>(
    index: usize,
    observation: &DerivedSourceObservation,
    mut poll: impl FnMut() -> Result<(), E>,
) -> Result<Vec<CanonicalBasisEntry>, E> {
    struct Locator<'a, E, F> {
        text: String,
        poll: &'a mut F,
        error: Option<E>,
    }
    impl<E, F: FnMut() -> Result<(), E>> std::fmt::Write for Locator<'_, E, F> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            if let Err(error) = (self.poll)() {
                self.error = Some(error);
                return Err(std::fmt::Error);
            }
            self.text.push_str(text);
            Ok(())
        }
    }
    let mut locator = Locator {
        text: String::new(),
        poll: &mut poll,
        error: None,
    };
    if observation.key().write_locator(&mut locator).is_err() {
        return Err(locator
            .error
            .expect("the String locator writer refuses only its liveness callback"));
    }
    let mut entries = Vec::new();
    let prefix = format!("fact.{index}");
    push(
        &mut entries,
        format!("{prefix}.locator"),
        CanonicalBasisEntryKind::Locator,
        text(locator.text),
    );
    append(&mut entries, &prefix, observation, &mut poll)?;
    Ok(entries)
}
