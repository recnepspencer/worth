use super::semantic_encoding::framed_record;
use super::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramRuleDeclaration, ValidatedApplicationProgram,
};

/// Canonically ordered, read-only description of a validated program.
///
/// Records are diagnostic evidence only. They carry no native type identity,
/// installation admission, execution handle, or publication authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramManifest {
    program_identity: String,
    records: Box<[String]>,
}

impl ApplicationProgramManifest {
    pub fn program_identity(&self) -> &str {
        &self.program_identity
    }

    pub fn records(&self) -> &[String] {
        &self.records
    }
}

impl<Schema, Program> ValidatedApplicationProgram<Schema, Program> {
    pub fn normalized_manifest(&self) -> ApplicationProgramManifest {
        ApplicationProgramManifest::normalize(
            self.identity().as_str(),
            self.features(),
            self.actions(),
            self.connections(),
            self.rules(),
        )
    }
}

impl ApplicationProgramManifest {
    /// Normalizes the declared parts of one validated program. Validation mints
    /// the canonical revision from exactly these records, so the manifest and
    /// the revision can never describe different meaning.
    pub(super) fn normalize(
        program_identity: &str,
        features: &[ApplicationFeatureDeclaration],
        actions: &[ApplicationActionDeclaration],
        connections: &[ApplicationConnectionDeclaration],
        rules: &[ApplicationProgramRuleDeclaration],
    ) -> Self {
        let mut records = Vec::new();
        for feature in features {
            let owner = feature.composition_instance();
            records.push(framed_record(
                "feature",
                [
                    ("owner", owner.to_owned()),
                    ("identity", feature.identity().to_owned()),
                    ("major", feature.major().to_string()),
                    ("minor", feature.minor().to_string()),
                ],
            ));
            records.extend(feature.inputs().iter().map(|input| {
                framed_record(
                    "port",
                    [
                        ("owner", owner.to_owned()),
                        ("feature", feature.identity().to_owned()),
                        ("direction", "input".to_owned()),
                        ("identity", input.identity().to_owned()),
                        ("required", input.required().to_string()),
                    ],
                )
            }));
            records.extend(feature.outputs().iter().map(|output| {
                framed_record(
                    "port",
                    [
                        ("owner", owner.to_owned()),
                        ("feature", feature.identity().to_owned()),
                        ("direction", "output".to_owned()),
                        ("identity", output.identity().to_owned()),
                    ],
                )
            }));
            for artifact in feature.derived_artifacts() {
                let mut dependencies = artifact
                    .dependencies()
                    .iter()
                    .map(|dependency| dependency.identity())
                    .collect::<Vec<_>>();
                dependencies.sort_unstable();
                let resources = artifact.resource_ceiling();
                records.push(framed_record(
                    "artifact",
                    [
                        ("owner", owner.to_owned()),
                        ("feature", feature.identity().to_owned()),
                        ("identity", artifact.identity().to_owned()),
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
                            "reconstruction",
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
                        ("work", resources.maximum_work().to_string()),
                        ("bytes", resources.maximum_retained_bytes().to_string()),
                        ("stopped", artifact.stopped_outcome().to_owned()),
                    ]),
                ));
            }
            for collection in feature.derived_collections() {
                records.push(framed_record(
                    "collection",
                    [
                        ("owner", owner.to_owned()),
                        ("feature", feature.identity().to_owned()),
                        ("identity", collection.identity().to_owned()),
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
                    ],
                ));
            }
            for computation in feature.managed_computations() {
                let resources = computation.resources();
                records.push(framed_record(
                    "computation",
                    [
                        ("owner", owner.to_owned()),
                        ("feature", feature.identity().to_owned()),
                        ("identity", computation.identity().to_owned()),
                        ("input", computation.input().to_owned()),
                        ("output", computation.output_artifact().to_owned()),
                        ("partition", computation.partition().to_owned()),
                        ("reuse", computation.reuse().to_owned()),
                        ("stopped", computation.stopped().to_owned()),
                        (
                            "execution",
                            computation.execution().canonical_token().to_owned(),
                        ),
                        ("ordering", computation.ordering().to_owned()),
                        ("work", resources.maximum_work().to_string()),
                        ("bytes", resources.maximum_retained_bytes().to_string()),
                    ],
                ));
            }
        }
        for action in actions {
            records.push(framed_record(
                "action",
                [
                    ("owner", action.composition_instance().to_owned()),
                    ("feature", action.feature().to_owned()),
                    (
                        "input",
                        action.operation_input_identity().as_str().to_owned(),
                    ),
                    ("conditional", action.conditional_only().to_string()),
                    (
                        "required-output",
                        action.required_output_source().to_string(),
                    ),
                    ("binding", action.binding().to_owned()),
                    (
                        "invariant",
                        action
                            .evaluated_requirement()
                            .map_or("", |attachment| attachment.identity())
                            .to_owned(),
                    ),
                    (
                        "observation",
                        action
                            .correspondence()
                            .map_or("", |attachment| attachment.identity())
                            .to_owned(),
                    ),
                    (
                        "locality",
                        action
                            .locality()
                            .map_or("", |locality| locality.identity())
                            .to_owned(),
                    ),
                    (
                        "granule",
                        action
                            .locality()
                            .map_or("", |locality| locality.granule().canonical_token())
                            .to_owned(),
                    ),
                    (
                        "change",
                        action
                            .change_shape()
                            .map_or("", |change| change.identity())
                            .to_owned(),
                    ),
                    (
                        "posture",
                        action
                            .change_shape()
                            .map_or("", |change| change.posture().canonical_token())
                            .to_owned(),
                    ),
                    (
                        "effect",
                        action
                            .external_input()
                            .map_or("", |attachment| attachment.identity())
                            .to_owned(),
                    ),
                ],
            ));
        }
        records.extend(connections.iter().map(|connection| {
            framed_record(
                "connection",
                [
                    ("identity", connection.identity().to_owned()),
                    ("source-instance", connection.source_instance().to_owned()),
                    ("source-feature", connection.source_feature().to_owned()),
                    ("source-port", connection.source_port().to_owned()),
                    ("target-instance", connection.target_instance().to_owned()),
                    ("target-feature", connection.target_feature().to_owned()),
                    ("target-port", connection.target_port().to_owned()),
                    ("required", connection.target_required().to_string()),
                    (
                        "exported",
                        connection.exports_across_instances().to_string(),
                    ),
                ],
            )
        }));
        records.extend(rules.iter().map(|rule| {
            framed_record(
                "invariant",
                [
                    ("owner", rule.composition_instance().to_owned()),
                    ("identity", rule.identity().to_owned()),
                    ("major", rule.major().to_string()),
                    ("minor", rule.minor().to_string()),
                    ("point", rule.execution_point().canonical_token().to_owned()),
                    ("local-owner", rule.local_owner().unwrap_or("").to_owned()),
                ],
            )
        }));
        records.sort_unstable();
        Self {
            program_identity: program_identity.to_owned(),
            records: records.into_boxed_slice(),
        }
    }
}
