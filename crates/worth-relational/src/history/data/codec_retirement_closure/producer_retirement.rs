//! Independently reconstruct a serialized succession closure from its before image.
use super::{charge, source_scope_closure, CodecRetirementClosureStop as Stop};
use crate::history::data::canonical_commit_envelope::{
    OutputOriginHistory, ValidatedOutputOrigins,
};
use crate::{
    mvcc::RequiredOutputRolePosture as Posture,
    publication::patch::data::RecordStructuralChange as Change, runtime::RelationalRuntime,
    transactions::data::RecordRef,
};
use worth_execution::MapKernelContext;

pub(super) fn validate(
    runtime: &RelationalRuntime,
    view: ValidatedOutputOrigins<'_>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<(), Stop> {
    let envelope = view.envelope();
    let Some(event) = &envelope.producer_cascade_retirement else {
        return Ok(());
    };
    let root = envelope
        .required_output_origin
        .as_ref()
        .ok_or("producer retirement lost generator")?;
    let predecessor = root
        .publication
        .predecessor()
        .ok_or("producer retirement lost generator predecessor")?;
    let previous = validated_history
        .by_commit(
            predecessor,
            &mut |commit| runtime.history.canonical_envelope(commit),
            context,
        )?
        .ok_or("producer retirement lost canonical predecessor")?;
    let prior = previous.output_origin_for_route(&root.meaning, context)?;
    charge(
        context,
        prior
            .members
            .len()
            .saturating_mul(envelope.patch.authoritative_record_patches.len()),
    )?;
    let anchors = prior
        .members
        .iter()
        .filter(|m| {
            m.posture != Posture::Retire
                && envelope.patch.authoritative_record_patches.iter().any(|p| {
                    p.target == RecordRef::Entity(m.id) && p.structural_change == Change::Deleted
                })
        })
        .map(|m| m.id)
        .collect::<Vec<_>>();
    if anchors != event.anchors {
        return Err(
            "producer retirement anchors differ from canonical predecessor deletions".into(),
        );
    }
    let latest = source_scope_closure::historical_positions(
        envelope.commit.parents.first().copied(),
        runtime.history.catalog_len(),
        None,
        &mut |commit| runtime.history.canonical_envelope(commit),
        &mut |event, context| {
            if event.required_output_materialization.is_some() {
                super::materialization::validate_semantics(runtime, event, context, None, schema)?;
            }
            Ok(())
        },
        context,
    )?;
    let expected = source_scope_closure::select(&anchors, &latest, context)?;
    charge(
        context,
        expected.len().saturating_mul(event.publications.len() + 1),
    )?;
    if expected.len() != event.publications.len() {
        return Err("producer retirement omits or adds a closure publication".into());
    }
    for selected in expected {
        let mut retired = None;
        for publication in &event.publications {
            charge(
                context,
                publication.meaning.family.len() + selected.origin.meaning.family.len() + 65,
            )?;
            if publication.meaning.same_route(&selected.origin.meaning) {
                retired = Some(publication);
                break;
            }
        }
        let retired = retired.ok_or("producer retirement contains a foreign route")?;
        if retired.publication.predecessor() != Some(selected.origin.commit.commit_id) {
            return Err("producer retirement changes its selected predecessor".into());
        }
        super::publication_succession::validate_route(runtime, view, &selected.origin, context)?;
        for edge in &selected.origin.relations {
            charge(context, envelope.patch.authoritative_record_patches.len())?;
            if !envelope.patch.authoritative_record_patches.iter().any(|p| {
                p.target == RecordRef::Relation(edge.id) && p.structural_change == Change::Deleted
            }) {
                return Err("producer retirement leaves an owned relation live".into());
            }
        }
    }
    Ok(())
}
