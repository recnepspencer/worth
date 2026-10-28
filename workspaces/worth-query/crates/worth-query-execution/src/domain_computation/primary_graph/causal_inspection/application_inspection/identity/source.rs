//! Source identity describes the actual canonical commit and original binding.
use super::*;
pub(super) fn identities<Stop>(
    target: &Arc<WorthQueryApplicationInspectionTarget>,
    session: &WorthQueryManagedGraphWorkSession,
    current: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<InspectionReferenceIdentities, InspectionIdentityStop<Stop>> {
    let original = target
        .source()
        .ok_or(InspectionIdentityStop::ForeignSession)?;
    let schema = original.disclosure.schema();
    let publication = &original.publication;
    let commit = original.changes.commit_reference();
    let observation = compose(
        "query.application-inspection.performed-source.v1",
        &[
            Part::Number(original.runtime),
            Part::Number(schema.runtime_ordinal()),
            Part::Number(schema.generation()),
            Part::Digest(*schema.package_identity()),
            Part::Digest(*schema.schema_identity()),
            Part::Digest(CanonicalDigestId::new(original.operation)),
            Part::Digest(*target.contract().graph_obligations().identity().digest()),
            Part::Text(
                original
                    .disclosure
                    .association()
                    .declaration()
                    .parts()
                    .identity
                    .as_str(),
            ),
            Part::Digest(*original.disclosure.disclosure().query().digest()),
            Part::Digest(target.parameters()),
            Part::Number(publication.branch_identity().owner_identity().get()),
            Part::Text(publication.branch_identity().name().as_str()),
            Part::Number(publication.lifecycle_incarnation().ordinal()),
            Part::Number(publication.reference_generation().get()),
            Part::Number(publication.selected_commit().owner_identity().get()),
            Part::Number(publication.selected_commit().ordinal()),
            Part::Number(commit.commit_id.0),
            Part::Number(commit.version_id.0),
            Part::Text("performed-source"),
        ],
        admit,
    )?;
    let admission = compose(
        "query.application-inspection.source-admission.v1",
        &[
            Part::Number(session.runtime_authority().as_u64()),
            Part::Number(session.identity().as_u64()),
            Part::Number(session.managed_run_identity().as_u64()),
            Part::Number(session.plan_identity().as_u64()),
            Part::Number(current.runtime_instance_id()),
            Part::Text(current.branch_id().0.as_str()),
            Part::Number(current.snapshot_id().0),
        ],
        admit,
    )?;
    let anchor = compose(
        "query.application-inspection.source-anchor.v1",
        &[
            Part::Text(observation.terminal_projection_for_reporting()),
            Part::Text(admission.terminal_projection_for_reporting()),
            Part::Text("why-performed"),
            Part::Text("reference-only"),
        ],
        admit,
    )?;
    Ok(InspectionReferenceIdentities {
        observation,
        admission,
        anchor,
    })
}
