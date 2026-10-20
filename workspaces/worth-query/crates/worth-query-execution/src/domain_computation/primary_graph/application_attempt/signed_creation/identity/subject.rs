//! Certified coverage binds the stored source-to-snapshot association, not transient admission reads.
use super::{
    AspectFieldLocator, AspectValue, BTreeMap, Denial, EntityId, IdentityEntries,
    WorthQueryCanonicalWorkEvidence,
};
use worth_query_declaration::facade::application_schema::ApplicationSigningSubjectDependency;

pub(in crate::domain_computation::primary_graph::application_attempt::signed_creation) fn certified_subject(
    fields: &BTreeMap<AspectFieldLocator, AspectValue>,
    scope: EntityId,
    dependencies: &[ApplicationSigningSubjectDependency],
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), Denial> {
    super::check_interruption(context)?;
    let mut entries = IdentityEntries::new();
    entries.entity("scope", scope);
    entries.fields("subject", fields, context)?;
    entries.number("certified-dependency-count", dependencies.len() as u64);
    for (index, dependency) in dependencies.iter().enumerate() {
        super::check_interruption(context)?;
        let prefix = format!("certified-dependency.{index}");
        entries.number(format!("{prefix}.partition"), dependency.source.partition);
        entries.number(format!("{prefix}.slot"), dependency.source.slot);
        entries.number(
            format!("{prefix}.generation"),
            u64::from(dependency.source.generation),
        );
        for (name, field) in [
            ("source", &dependency.source_field),
            ("capture", &dependency.captured_field),
        ] {
            entries.text(format!("{prefix}.{name}.entity"), field.entity());
            entries.text(format!("{prefix}.{name}.aspect"), field.aspect());
            entries.text(format!("{prefix}.{name}.field"), field.field());
        }
        entries.text(format!("{prefix}.value-binding"), &dependency.value_binding);
        entries.number(
            format!("{prefix}.native-revision"),
            dependency.native_revision,
        );
    }
    entries.derive("worth-query-signed-certified-subject-v2", request, context)
}
