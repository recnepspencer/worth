//! Follow the selected producer antecedent, never the displaced current publication.
use super::{charge, CodecRetirementClosureStop as Stop, RelationalRuntime};
use crate::history::data::canonical_commit_envelope::{
    OutputOriginHistory, OwnedValidatedOutputOrigins,
};
use crate::{
    history::data::CanonicalOutputPublication, mvcc::RequiredOutputRolePosture as Posture,
};
use worth_execution::MapKernelContext;

pub(crate) fn antecedent(
    runtime: &RelationalRuntime,
    envelope: &OwnedValidatedOutputOrigins,
    origin: &crate::history::data::CanonicalRequiredOutputOrigin,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<OwnedValidatedOutputOrigins, Stop> {
    let CanonicalOutputPublication::HistoricalSelection {
        selected,
        generation,
        ..
    } = origin.publication
    else {
        return Err("historical reconstruction requires a selection origin".into());
    };
    let previous = validated_history
        .by_commit(
            selected,
            &mut |commit| runtime.history.canonical_envelope(commit),
            context,
        )?
        .ok_or("historical producer antecedent is unavailable")?;
    if previous.commit.version_id != generation || selected == envelope.commit.commit_id {
        return Err("historical producer antecedent generation differs".into());
    }
    let original = previous.output_origin_for_route(&origin.meaning, context)?;
    charge(
        context,
        original.producer.len()
            + origin.producer.len()
            + original.members.len()
            + origin.members.len()
            + original.relations.len()
            + origin.relations.len()
            + 1,
    )?;
    if !original.source_scope_owned
        || original.meaning != origin.meaning
        || original.producer != origin.producer
        || original.dependency_identity != origin.dependency_identity
        || original.birth != origin.birth
        || original.birth_producer != origin.birth_producer
        || original.birth_dependency != origin.birth_dependency
        || original
            .members
            .iter()
            .filter(|m| m.posture != Posture::Retire)
            .count()
            != origin.members.len()
        || original.relations.len() != origin.relations.len()
    {
        return Err("historical selection changes producer antecedent meaning".into());
    }
    for (member, old) in origin.members.iter().zip(
        original
            .members
            .iter()
            .filter(|m| m.posture != Posture::Retire),
    ) {
        charge(
            context,
            member.role.len()
                + old.role.len()
                + member.entity_name.len()
                + old.entity_name.len()
                + 1,
        )?;
        if member.id != old.id
            || member.kind != old.kind
            || member.role != old.role
            || member.entity_name != old.entity_name
            || member.birth != old.birth
            || member.birth_producer != old.birth_producer
            || member.birth_dependency != old.birth_dependency
            || member.posture != Posture::Preserve
            || member.after_write != envelope.commit.version_id
        {
            return Err("historical selection invents or remaps a producer member".into());
        }
    }
    for (edge, old) in origin.relations.iter().zip(&original.relations) {
        charge(context, 1)?;
        if (edge.id, edge.kind, edge.source, edge.target, edge.birth)
            != (old.id, old.kind, old.source, old.target, old.birth)
            || edge.after_write != envelope.commit.version_id
        {
            return Err("historical selection invents or remaps a producer relation".into());
        }
    }
    Ok(previous)
}
