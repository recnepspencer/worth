//! Full graph truth; live issuance provenance and payment are deliberately absent.
use super::*;
use crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation;
pub(super) fn append_values(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    observation: &StructuralWalkObservation,
) {
    match append(entries, prefix, observation, &mut || {
        Ok::<(), std::convert::Infallible>(())
    }) {
        Ok(()) => (),
        Err(never) => match never {},
    }
}
pub(super) fn append<E>(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    observation: &StructuralWalkObservation,
    poll: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    poll()?;
    kind(entries, prefix, "complete-live-structural-walk");
    let query = observation.key();
    number_u64(
        entries,
        prefix,
        "entity-kind",
        u64::from(query.entity_kind().as_u32()),
    );
    number_u64(
        entries,
        prefix,
        "relation-kind",
        u64::from(query.relation_kind().as_u32()),
    );
    push(
        entries,
        format!("{prefix}.relation"),
        CanonicalBasisEntryKind::Identity,
        text(query.relation()),
    );
    push(
        entries,
        format!("{prefix}.direction"),
        CanonicalBasisEntryKind::Value,
        text(match query.direction() {
            worth_query_declaration::facade::application_schema::RelationDirection::Forward => {
                "forward"
            }
            worth_query_declaration::facade::application_schema::RelationDirection::Reverse => {
                "reverse"
            }
        }),
    );
    number(entries, prefix, "root-count", query.roots().len());
    for (index, id) in query.roots().iter().enumerate() {
        poll()?;
        entity(entries, prefix, &format!("root.{index}"), *id);
    }
    number(
        entries,
        prefix,
        "visit-count",
        observation.native().visits().len(),
    );
    for (index, visit) in observation.native().visits().iter().enumerate() {
        poll()?;
        let item = format!("{prefix}.visit.{index}");
        entity(entries, &item, "entity", visit.entity.entity_id);
        number_u64(
            entries,
            &item,
            "kind",
            u64::from(visit.entity.kind_id.as_u32()),
        );
        number(entries, &item, "distance", visit.distance);
        push(
            entries,
            format!("{item}.lifecycle"),
            CanonicalBasisEntryKind::Value,
            text("live"),
        );
    }
    number(
        entries,
        prefix,
        "relation-count",
        observation.native().relations().len(),
    );
    for (index, edge) in observation.native().relations().iter().enumerate() {
        poll()?;
        let item = format!("{prefix}.relation.{index}");
        record(entries, &item, "id", edge.relation_id);
        entity(entries, &item, "source", edge.source);
        entity(entries, &item, "target", edge.target);
        number_u64(entries, &item, "kind", u64::from(edge.kind_id.as_u32()));
        push(
            entries,
            format!("{item}.lifecycle"),
            CanonicalBasisEntryKind::Value,
            text("live"),
        );
    }
    poll()?;
    Ok(())
}

pub(in super::super) fn entries_checked<E>(
    index: usize,
    observation: &StructuralWalkObservation,
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
