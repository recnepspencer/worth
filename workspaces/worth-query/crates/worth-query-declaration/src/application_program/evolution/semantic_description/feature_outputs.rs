//! Canonical output/resource facts for a feature's derived meanings.

use super::{
    fact, framed_fields, framed_parts, ApplicationFeatureDeclaration, ApplicationSemanticFact,
    ApplicationSemanticFamily,
};

pub(super) fn append_feature_outputs(
    facts: &mut Vec<ApplicationSemanticFact>,
    feature: &ApplicationFeatureDeclaration,
) {
    let owner = feature.composition_instance();
    let feature_identity = feature.identity();
    for artifact in feature.derived_artifacts() {
        let subject = framed_parts(&[owner, feature_identity, "artifact", artifact.identity()]);
        let mut dependencies = artifact
            .dependencies()
            .iter()
            .map(|dependency| dependency.identity())
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            framed_fields(
                [
                    ("output", artifact.output().to_owned()),
                    ("locality", artifact.locality().identity().to_owned()),
                    (
                        "granule",
                        artifact.locality().granule().canonical_token().to_owned(),
                    ),
                    (
                        "retention",
                        artifact.retention().canonical_token().to_owned(),
                    ),
                    (
                        "succession",
                        artifact.succession().canonical_token().to_owned(),
                    ),
                    ("required", artifact.required().to_string()),
                    ("producer", artifact.producer_family().to_owned()),
                ]
                .into_iter()
                .chain(
                    dependencies
                        .into_iter()
                        .map(|dependency| ("dependency", dependency.to_owned())),
                )
                .chain([
                    ("reuse", artifact.reuse_rule().to_owned()),
                    ("stopped", artifact.stopped_outcome().to_owned()),
                ]),
            ),
        ));
        let resources = artifact.resource_ceiling();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            framed_fields([
                (
                    "work",
                    resources
                        .maximum_work()
                        .map_or_else(|| "absent".to_owned(), |work| work.to_string()),
                ),
                (
                    "bytes",
                    resources
                        .maximum_retained_bytes()
                        .map_or_else(|| "absent".to_owned(), |bytes| bytes.to_string()),
                ),
            ]),
        ));
    }
    for collection in feature.derived_collections() {
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            framed_parts(&[owner, feature_identity, "collection", collection.identity()]),
            framed_fields([
                ("contributor", collection.contributor().to_owned()),
                ("grouping", collection.grouping().to_owned()),
                ("measures", collection.measures().to_owned()),
                ("lineage", collection.lineage().to_owned()),
                ("applicability", collection.applicability().to_owned()),
                (
                    "incomplete",
                    collection.incomplete().canonical_token().to_owned(),
                ),
                ("incremental", collection.incremental_update().to_owned()),
            ]),
        ));
    }
    for computation in feature.managed_computations() {
        let subject = framed_parts(&[
            owner,
            feature_identity,
            "computation",
            computation.identity(),
        ]);
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            framed_fields([
                ("input", computation.input().to_owned()),
                ("output", computation.output_artifact().to_owned()),
                ("partition", computation.partition().to_owned()),
                ("reuse", computation.reuse().to_owned()),
                ("stopped", computation.stopped().to_owned()),
                (
                    "execution",
                    computation.execution().canonical_token().to_owned(),
                ),
                (
                    "determinism",
                    super::super::super::managed_computation::determinism_canonical_token(
                        computation.determinism(),
                    ),
                ),
            ]),
        ));
        let resources = computation.resources();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            framed_fields([
                ("work", resources.maximum_work().to_string()),
                ("bytes", resources.maximum_retained_bytes().to_string()),
            ]),
        ));
    }
}
