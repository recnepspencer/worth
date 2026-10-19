//! V3 coverage includes optional observations and the native identity chain of each capture.
use super::{
    AspectFieldLocator, AspectValue, BTreeMap, Denial, EntityId, IdentityEntries,
    WorthQueryCanonicalWorkEvidence,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryRootPathDirection as Direction,
    application_schema::{
        ApplicationSigningEntityCoordinates as Coordinates,
        ApplicationSigningGraphSubjectDependency as Dependency,
        ApplicationSigningSubjectObservedFieldVersion as Version,
        ApplicationSigningSubjectSelectionDependency as Selection,
    },
};
pub(in crate::domain_computation::primary_graph::application_attempt::signed_creation) fn certified_graph_subject(
    fields: &BTreeMap<AspectFieldLocator, AspectValue>,
    scope: EntityId,
    dependencies: &[Dependency],
    selections: &[Selection],
    maximum_bytes: usize,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), Denial> {
    let mut entries = IdentityEntries::new();
    entries.entity("scope", scope);
    entries.fields("subject", fields, context)?;
    entries.number("certified-dependency-count", dependencies.len() as u64);
    for (index, dependency) in dependencies.iter().enumerate() {
        super::admit_assembly(
            context,
            dependency
                .maximum_encoded_bytes()
                .ok_or(Denial::RepresentationCapacity)?,
        )?;
        let prefix = format!("certified-dependency.{index}");
        coordinates(&mut entries, &prefix, dependency.source);
        for (name, field) in [
            ("source", &dependency.source_field),
            ("capture", &dependency.captured_field),
        ] {
            entries.text(format!("{prefix}.{name}.entity"), field.entity());
            entries.text(format!("{prefix}.{name}.aspect"), field.aspect());
            entries.text(format!("{prefix}.{name}.field"), field.field());
        }
        entries.text(format!("{prefix}.value-binding"), &dependency.value_binding);
        let (presence, revision) = match dependency.observed_version {
            Version::Present { native_revision } => ("present", Some(native_revision)),
            Version::Absent { native_revision } => ("absent", native_revision),
        };
        entries.text(format!("{prefix}.presence"), presence);
        entries.text(
            format!("{prefix}.revision-kind"),
            if revision.is_some() {
                "observed"
            } else {
                "absent-aspect"
            },
        );
        if let Some(revision) = revision {
            entries.number(format!("{prefix}.native-revision"), revision);
        }
    }
    entries.number("certified-selection-count", selections.len() as u64);
    for (index, selection) in selections.iter().enumerate() {
        super::admit_assembly(
            context,
            selection
                .maximum_encoded_bytes()
                .ok_or(Denial::RepresentationCapacity)?,
        )?;
        let prefix = format!("certified-selection.{index}");
        coordinates(&mut entries, &format!("{prefix}.anchor"), selection.anchor);
        coordinates(
            &mut entries,
            &format!("{prefix}.selected"),
            selection.selected,
        );
        entries.text(
            format!("{prefix}.capture.entity"),
            selection.captured_field.entity(),
        );
        entries.text(
            format!("{prefix}.capture.aspect"),
            selection.captured_field.aspect(),
        );
        entries.text(
            format!("{prefix}.capture.field"),
            selection.captured_field.field(),
        );
        entries.text(format!("{prefix}.relation"), &selection.relation);
        entries.text(
            format!("{prefix}.direction"),
            match selection.direction {
                Direction::Forward => "forward",
                Direction::Reverse => "reverse",
            },
        );
        entries.number(
            format!("{prefix}.native-revision"),
            selection.native_revision,
        );
    }
    entries.derive(
        "worth-query-signed-certified-graph-subject-v3",
        maximum_bytes,
        request,
        context,
    )
}
fn coordinates(entries: &mut IdentityEntries, prefix: &str, coordinates: Coordinates) {
    entries.number(format!("{prefix}.partition"), coordinates.partition);
    entries.number(format!("{prefix}.slot"), coordinates.slot);
    entries.number(
        format!("{prefix}.generation"),
        u64::from(coordinates.generation),
    );
}
