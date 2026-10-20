//! Derive durable subject correspondence from installed snapshot initializers, never all admission facts.
use super::super::WorthQueryApplicationObservedFact as Fact;
use super::{
    identity,
    sealing::{BodyCandidate, WorthQuerySignedCreationPreparationDenial as Denial},
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
        ApplicationSchema, ApplicationSigningEntityCoordinates,
        ApplicationSigningProvenanceEncoding as Encoding,
        ApplicationSigningSubjectDependency as Dependency,
    },
};

pub(super) fn creation<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    candidate: &BodyCandidate<Schema, Binding>,
) -> Result<(&ApplicationOperationBodyCreation, Encoding), Denial> {
    let creation = candidate
        .read_set
        .admission
        .allowed_graph_contract()
        .body()
        .and_then(|body| body.creation())
        .ok_or(Denial::CandidateMismatch)?;
    let provenance = creation
        .signing()
        .ok_or(Denial::CandidateMismatch)?
        .provenance();
    let encoding = creation
        .fields()
        .iter()
        .find_map(|field| match field {
            Field::SigningProvenance { field, binding } if field == provenance => {
                Encoding::for_binding(binding)
            }
            _ => None,
        })
        .ok_or(Denial::CandidateMismatch)?;
    Ok((creation, encoding))
}

pub(super) fn capture<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    candidate: &BodyCandidate<Schema, Binding>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Vec<Dependency>, Denial> {
    // O((captures + retained facts) log captures), with borrowed native keys.
    // The admitted correspondence workspace covers this transient lookup; no
    // other document, persistent reverse index or historical record is scanned.
    let (creation, encoding) = creation::<Schema, Binding>(candidate)?;
    if encoding == Encoding::AuthenticationV1 {
        return Ok(Vec::new());
    }
    let subject = candidate.read_set.admission.scope_entity_id();
    let mut revisions = std::collections::BTreeMap::new();
    for (source, _, _) in creation.fields().iter().filter_map(scope_capture) {
        identity::check_interruption(context)?;
        let locator = candidate
            .read_set
            .lease
            .layout
            .field_locator(source.entity(), source.aspect(), source.field())
            .ok_or(Denial::CandidateMismatch)?;
        revisions.insert((locator.aspect().aspect_key(), locator.field_path()), None);
    }
    for fact in candidate.read_set.facts.iter() {
        identity::check_interruption(context)?;
        if let Fact::SourceFieldRevision {
            entity_id,
            locator,
            native_revision,
        } = fact
            && *entity_id == subject
            && let Some(revision) =
                revisions.get_mut(&(locator.aspect().aspect_key(), locator.field_path()))
        {
            *revision = *native_revision;
        }
    }
    let mut dependencies =
        Vec::with_capacity(creation.fields().iter().filter_map(scope_capture).count());
    for (source, captured, binding) in creation.fields().iter().filter_map(scope_capture) {
        let locator = candidate
            .read_set
            .lease
            .layout
            .field_locator(source.entity(), source.aspect(), source.field())
            .ok_or(Denial::CandidateMismatch)?;
        let revision = revisions
            .get(&(locator.aspect().aspect_key(), locator.field_path()))
            .copied()
            .flatten()
            .filter(|revision| {
                revision.presence()
                    == worth_relational::facade::runtime::RelationalFieldPresence::Present
            })
            .ok_or(Denial::CandidateMismatch)?;
        dependencies.push(Dependency {
            source: ApplicationSigningEntityCoordinates {
                partition: subject.partition_value_u64(),
                slot: subject.local_slot_value(),
                generation: subject.generation_value(),
            },
            source_field: source.clone(),
            captured_field: captured.clone(),
            value_binding: binding.as_str().to_owned(),
            native_revision: revision.version().0,
        });
    }
    Ok(dependencies)
}

fn scope_capture(
    field: &Field,
) -> Option<(
    &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    &worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
)> {
    match field {
        Field::CapturedScopeField {
            field,
            source,
            binding,
            role: worth_query_declaration::facade::application_schema::ApplicationOperationBodyCaptureRole::Subject,
            ..
        } => Some((source, field, binding)),
        Field::Literal { .. }
        | Field::Input { .. }
        | Field::OptionalInput { .. }
        | Field::SigningProvenance { .. }
        | Field::PrincipalIdentity { .. } => None,
        Field::RelatedField { .. } => None,
        Field::ScopeField { .. } | Field::PresentScopeField { .. } | Field::CapturedScopeField { .. } => None,
    }
}
