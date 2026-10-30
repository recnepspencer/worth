use super::*;
pub fn retained_empty_and_absent_bodies_keep_distinct_rich_outcomes() {
    use WorthQueryApplicationEvidenceFamily as Family;
    use WorthQueryMaterializedFamilyOutcome as Outcome;
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let access = || journey::access(&application, &principal, &scope);
    let request = application.runtime().request(&principal, &scope);
    for retain in [false, true] {
        let (original, rows) = read!(application, &principal, &scope, Read, false, retain);
        assert!(rows.is_empty());
        let rich = request
            .inspect::<Binding>(&original)
            .why_observed()
            .materialize(
                WorthQueryApplicationMaterializationRequest::family(Family::QueryInspection)
                    .with_source_effects()
                    .with_producer_evidence(),
            )
            .execute_governed(access(), access())
            .unwrap();
        let body = rich.materialization();
        assert_eq!(
            body.source_outcome(),
            Outcome::RequestedButNotProvidedBySelectedObservationMode
        );
        assert_eq!(
            body.producer_outcome(),
            Outcome::RequestedButNotProvidedBySelectedObservationMode
        );
        assert!(body.source_effects().is_none());
        assert!(body.producer_evaluation().is_none());
        assert!(body.producer_connection().is_none());
        if retain {
            assert_eq!(
                body.family_outcome(Family::QueryInspection),
                Outcome::Provided
            );
            assert_eq!(body.original_query().unwrap().rows().len(), 0);
        } else {
            assert_eq!(
                body.family_outcome(Family::QueryInspection),
                Outcome::OriginalProjectionNotRetained
            );
            assert!(body.original_query().is_none());
        }
        rich.close();
        original.close();
        let (original, rows) = read!(application, &principal, &scope, FieldRead, false, retain);
        assert!(rows.is_empty());
        let denied = request
            .inspect::<FieldBinding>(&original)
            .why_observed()
            .materialize(WorthQueryApplicationMaterializationRequest::family(
                Family::QueryInspection,
            ))
            .execute_governed(access(), access())
            .err()
            .expect("current field policy refuses retained and absent empty originals");
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationInspectionStopKind::Admission
        );
        assert!(denied.materialization_refusal().is_none());
        drop(denied);
        original.close();
    }
    assert_eq!(readers.observe().active(), baseline);
}
