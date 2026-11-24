//! Exact source ancestry and native writes for an authenticated producer group.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::*;

pub(super) fn source_gap(
    source: &CanonicalCommitEnvelope,
    original: &OwnedValidatedOutputOrigins,
    origin: &crate::history::data::CanonicalRequiredOutputOrigin,
    lookup: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    maximum: usize,
    validated_history: &mut OutputOriginHistory,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let mut next = Some(source.commit.commit_id);
    for _ in 0..maximum {
        charge(context, 1)?;
        let id = next.ok_or("maintenance original is outside source ancestry")?;
        if id == original.commit.commit_id {
            return Ok(());
        }
        let envelope = lookup(id)?.ok_or("maintenance source ancestry absent")?;
        if envelope.commit.version_id > source.commit.version_id {
            return Err("maintenance source ancestry moves forward".into());
        }
        super::super::super::require_linear_history(&envelope)?;
        let mut replaced = false;
        for candidate in envelope.output_origins() {
            charge(
                context,
                candidate.meaning.family.len() + origin.meaning.family.len() + 65,
            )?;
            if candidate.meaning.same_route(&origin.meaning) {
                replaced = true;
                break;
            }
        }
        if replaced
            || envelope
                .required_output_codec_retirements
                .iter()
                .any(|candidate| candidate.meaning.same_route(&origin.meaning))
        {
            return Err("maintenance original producer was superseded at its source".into());
        }
        for record in origin.outputs.iter().copied().map(RecordRef::Entity).chain(
            origin
                .relations
                .iter()
                .map(|relation| RecordRef::Relation(relation.id)),
        ) {
            let wrote = match record {
                RecordRef::Entity(id) => {
                    super::super::super::membership::writes(&envelope, id, context)?
                }
                RecordRef::Relation(id) => {
                    super::super::super::publication_succession::relation_writes(
                        &envelope, id, context,
                    )?
                }
            };
            if wrote {
                let event = envelope
                    .required_output_materialization
                    .as_ref()
                    .ok_or("ordinary native writer ended materialization producer continuity")?;
                if !event
                    .producer_publications
                    .contains(&original.commit.commit_id)
                    || !event.records.iter().any(|member| member.record == record)
                {
                    return Err("maintenance omitted a producer write".into());
                }
            }
        }
        if envelope.required_output_materialization.is_some() {
            validate_local(
                &envelope,
                lookup,
                maximum,
                validated_history,
                execution,
                context,
            )?;
        }
        next = envelope
            .required_output_materialization
            .as_ref()
            .map(|event| event.source_commit)
            .or_else(|| envelope.commit.parents.first().copied());
    }
    Err("maintenance source ancestry is incomplete or cyclic".into())
}

pub(super) fn source_write(
    source: &CanonicalCommitEnvelope,
    record: &RecordRef,
    lookup: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    maximum: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<VersionId, Stop> {
    let mut next = Some(source.commit.commit_id);
    for _ in 0..maximum {
        let envelope = lookup(next.ok_or("maintenance source write absent")?)?
            .ok_or("maintenance source write history absent")?;
        let writes = match *record {
            RecordRef::Entity(id) => {
                super::super::super::membership::writes(&envelope, id, context)?
            }
            RecordRef::Relation(id) => {
                super::super::super::publication_succession::relation_writes(
                    &envelope, id, context,
                )?
            }
        };
        if writes {
            return Ok(envelope.commit.version_id);
        }
        next = envelope.commit.parents.first().copied();
    }
    Err("maintenance source write ancestry incomplete or cyclic".into())
}
