//! Certified graph capture is sealed from the same native body observation and candidate.
mod observation;
mod selection;
use super::{identity, BodyCandidate, CapturedSubject, Denial};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus as Locus, ApplicationFieldPresence as Presence,
        ApplicationOperationBodyCaptureRole as Role, ApplicationOperationBodyCreatedField as Field,
        ApplicationOperationBodyCreation as Creation, ApplicationSchema,
        ApplicationSigningGraphSubjectDependency as Dependency,
        ApplicationSigningSubjectDependency as ScopeDependency,
        ApplicationSigningSubjectSelectionDependency as Selection,
    },
};

pub(super) fn quote(creation: &Creation) -> Result<usize, Denial> {
    creation
        .fields()
        .iter()
        .filter_map(capture_spec)
        .try_fold(128usize, |total, spec| {
            let fields = ScopeDependency::quote(spec.source, spec.target, spec.binding)?
                .checked_add(128 + size_of::<Dependency>())?;
            spec.path.into_iter().flat_map(|p| p.steps()).try_fold(
                total.checked_add(fields)?,
                |sum, hop| {
                    sum.checked_add(Selection::quote(spec.target, hop.relation())?)?
                        .checked_add(size_of::<Selection>())
                },
            )
        })
        .ok_or(Denial::RepresentationCapacity)
}

pub(super) fn capture<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    candidate: &BodyCandidate<Schema, Binding>,
    creation: &Creation,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<CapturedSubject, Denial> {
    identity::admit_assembly(context, quote(creation)?)?;
    let root = candidate.read_set.admission.scope_entity_id();
    let mut dependencies = Vec::new();
    let mut selections = Vec::new();
    for spec in creation.fields().iter().filter_map(capture_spec) {
        let source = selection::capture::<Schema, Binding>(
            candidate,
            root,
            &spec,
            request,
            context,
            &mut selections,
        )?;
        let observed_version =
            observation::capture::<Schema, Binding>(candidate, source, &spec, context)?;
        dependencies.push(Dependency {
            source: coordinates(source),
            source_field: spec.source.clone(),
            captured_field: spec.target.clone(),
            value_binding: spec.binding.to_owned(),
            observed_version,
        });
    }
    Ok(CapturedSubject::Graph {
        dependencies,
        selections,
    })
}

struct CaptureSpec<'a> {
    source: &'a Locus,
    target: &'a Locus,
    binding: &'a str,
    source_presence: Presence,
    target_presence: Presence,
    path: Option<
        &'a worth_query_declaration::facade::application_query::ApplicationQueryRootPathMeaning,
    >,
}
fn capture_spec(field: &Field) -> Option<CaptureSpec<'_>> {
    match field {
        Field::CapturedScopeField {
            field,
            source,
            binding,
            source_presence,
            role: Role::Subject,
            ..
        } => Some(CaptureSpec {
            source,
            target: field,
            binding: binding.as_str(),
            source_presence: *source_presence,
            target_presence: Presence::Required,
            path: None,
        }),
        Field::CapturedPathField {
            field,
            source,
            binding,
            target_presence,
            role: Role::Subject,
            ..
        } => Some(CaptureSpec {
            source: &source.field,
            target: field,
            binding: binding.as_str(),
            source_presence: source.presence,
            target_presence: *target_presence,
            path: Some(&source.path),
        }),
        _ => None,
    }
}
fn coordinates(
    entity: worth_relational::facade::identity::EntityId,
) -> worth_query_declaration::facade::application_schema::ApplicationSigningEntityCoordinates {
    worth_query_declaration::facade::application_schema::ApplicationSigningEntityCoordinates {
        partition: entity.partition_value_u64(),
        slot: entity.local_slot_value(),
        generation: entity.generation_value(),
    }
}
