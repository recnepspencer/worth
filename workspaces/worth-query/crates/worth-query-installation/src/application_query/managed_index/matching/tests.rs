//! Pure compiler-meaning twins; no runtime, session or READ is constructed.
use super::*;
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryParameterDefinition,
    ApplicationQueryPredicate,
};

struct Sequence;
worth_query_declaration::worth_query_portable_type!(Sequence => "worth.query.tests.predicate-sequence.v1");

fn definitions() -> (
    ErasedApplicationQueryDefinition,
    ErasedApplicationQueryDefinition,
) {
    let parent = crate::application_query::tests::definition_with_sequence_slot::<Sequence>(
        ApplicationQueryOrderingDirection::Ascending,
        "sequence",
    )
    .into_erased();
    let mut source = parent.clone().into_parts();
    source.parameters.clear();
    source.predicates.clear();
    let mut parent = parent.into_parts();
    let parameter = &parent.parameters[0];
    parent.parameters.push(ApplicationQueryParameterDefinition::from_untrusted_fields(
        "probe".into(), parameter.scalar_family(), worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(parameter.value_type().to_owned()), None, None,
    ));
    (
        ErasedApplicationQueryDefinition::from_untrusted_parts(source),
        ErasedApplicationQueryDefinition::from_untrusted_parts(parent),
    )
}
fn profile(
    source: &ErasedApplicationQueryDefinition,
    parent: &ErasedApplicationQueryDefinition,
) -> Result<WorthQueryManagedIndexPredicateProfile, WorthQueryManagedIndexPredicateProfileDenial> {
    let schema = CanonicalDigestId::new([1; 32]);
    let source = Graph::compile(
        source,
        &schema,
        CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap(),
    )
    .unwrap();
    let graph = Graph::compile(
        parent,
        &schema,
        CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap(),
    )
    .unwrap();
    tracked_profile(parent, "probe", &source, &graph)
}

#[test]
fn actual_compiled_root_projection_supports_explicit_conservative_refinement() {
    let (source, parent) = definitions();
    assert_eq!(
        profile(&source, &parent),
        Ok(WorthQueryManagedIndexPredicateProfile::ConservativeProjectedRefinement)
    );
    let mut direct = parent.into_parts();
    direct.predicates.clear();
    direct.parameters.retain(|p| p.name() == "probe");
    assert_eq!(
        profile(
            &source,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(direct)
        ),
        Ok(WorthQueryManagedIndexPredicateProfile::DirectSourceProjection)
    );
}

#[test]
fn unused_parameter_cannot_claim_a_tracked_predicate_profile() {
    let (source, parent) = definitions();
    let mut parts = parent.into_parts();
    let parameter = &parts.parameters[0];
    parts.parameters.push(ApplicationQueryParameterDefinition::from_untrusted_fields(
        "unused".into(), parameter.scalar_family(), worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(parameter.value_type().to_owned()), None, None,
    ));
    assert_eq!(
        profile(
            &source,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
        ),
        Err(WorthQueryManagedIndexPredicateProfileDenial::UntrackedParameters)
    );
}

#[test]
fn predicate_field_absent_from_source_projection_cannot_claim_tracked_refinement() {
    let (source, parent) = definitions();
    let mut parts = parent.into_parts();
    let old = &parts.predicates[0];
    parts.predicates = vec![ApplicationQueryPredicate::from_untrusted_fields(
        old.field().0.into(),
        old.field().1.into(),
        "unprojected".into(),
        old.parameter().into(),
        old.scalar_family(),
    )];
    assert_eq!(
        profile(
            &source,
            &ErasedApplicationQueryDefinition::from_untrusted_parts(parts)
        ),
        Err(WorthQueryManagedIndexPredicateProfileDenial::UntrackedSelection)
    );
}
