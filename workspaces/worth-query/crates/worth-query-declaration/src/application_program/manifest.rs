use super::ValidatedApplicationProgram;

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
        let mut records = Vec::new();
        for feature in self.features() {
            let owner = feature.composition_instance();
            records.push(format!(
                "feature|{owner}|{}|version={}.{}",
                feature.identity(),
                feature.major(),
                feature.minor()
            ));
            records.extend(feature.inputs().iter().map(|input| {
                format!(
                    "port|{owner}|{}|input|{}|required={}",
                    feature.identity(),
                    input.identity(),
                    input.required()
                )
            }));
            records.extend(feature.outputs().iter().map(|output| {
                format!(
                    "port|{owner}|{}|output|{}",
                    feature.identity(),
                    output.identity()
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
                records.push(format!(
                    "artifact|{owner}|{}|{}|output={}|locality={}:\
                     {:?}|retention={:?}|reconstruction={:?}|required={}|\
                     producer={}|dependencies={}|reuse={}|work={}|bytes={}|stopped={}",
                    feature.identity(),
                    artifact.identity(),
                    artifact.output(),
                    artifact.locality().identity(),
                    artifact.locality().granule(),
                    artifact.retention(),
                    artifact.succession(),
                    artifact.required(),
                    artifact.producer_family(),
                    dependencies.join(","),
                    artifact.reuse_rule(),
                    resources.maximum_work(),
                    resources.maximum_retained_bytes(),
                    artifact.stopped_outcome()
                ));
            }
            for collection in feature.derived_collections() {
                records.push(format!(
                    "collection|{owner}|{}|{}|contributor={}|grouping={}|measures={}|\
                     lineage={}|applicability={}|incomplete={:?}|incremental={}",
                    feature.identity(),
                    collection.identity(),
                    collection.contributor(),
                    collection.grouping(),
                    collection.measures(),
                    collection.lineage(),
                    collection.applicability(),
                    collection.incomplete(),
                    collection.incremental_update()
                ));
            }
            for computation in feature.managed_computations() {
                let resources = computation.resources();
                records.push(format!(
                    "computation|{owner}|{}|{}|input={}|output={}|partition={}|reuse={}|\
                     stopped={}|execution={:?}|ordering={}|work={}|bytes={}",
                    feature.identity(),
                    computation.identity(),
                    computation.input(),
                    computation.output_artifact(),
                    computation.partition(),
                    computation.reuse(),
                    computation.stopped(),
                    computation.execution(),
                    computation.ordering(),
                    resources.maximum_work(),
                    resources.maximum_retained_bytes()
                ));
            }
        }
        for action in self.actions() {
            records.push(format!(
                "action|{}|{}|{}|input={}|conditional={}|required-output={}|\
                 authority=binding:{}|invariant={}|observation={}|locality={}|change={}|effect={}",
                action.composition_instance(),
                action.feature(),
                action.binding(),
                action.operation_input_identity().as_str(),
                action.conditional_only(),
                action.required_output_source(),
                action.binding(),
                action
                    .evaluated_requirement()
                    .map_or("", |attachment| attachment.identity()),
                action
                    .correspondence()
                    .map_or("", |attachment| attachment.identity()),
                action.locality().map_or("", |locality| locality.identity()),
                action.change_shape().map_or("", |change| change.identity()),
                action
                    .external_input()
                    .map_or("", |attachment| attachment.identity())
            ));
        }
        records.extend(self.connections().iter().map(|connection| {
            format!(
                "connection|{}|{}:{}:{}|{}:{}:{}|required={}|exported={}",
                connection.identity(),
                connection.source_instance(),
                connection.source_feature(),
                connection.source_port(),
                connection.target_instance(),
                connection.target_feature(),
                connection.target_port(),
                connection.target_required(),
                connection.exports_across_instances()
            )
        }));
        records.extend(self.rules().iter().map(|rule| {
            format!(
                "invariant|{}|{}|version={}.{}|point={:?}|owner={}",
                rule.composition_instance(),
                rule.identity(),
                rule.major(),
                rule.minor(),
                rule.execution_point(),
                rule.local_owner().unwrap_or("")
            )
        }));
        records.sort_unstable();
        ApplicationProgramManifest {
            program_identity: self.identity().as_str().to_owned(),
            records: records.into_boxed_slice(),
        }
    }
}
