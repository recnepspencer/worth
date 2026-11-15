//! Capture owners answer exact complete ports of each original rostered program.
use super::WorthQueryAdmittedProgramSupport;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::PendingProducerRegistry,
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionDeclaration as Connection, ApplicationFeatureDeclaration as Feature,
    },
    application_schema::ApplicationSchema,
};

pub(in crate::domain_computation::primary_graph) struct ProgramInputPorts {
    features: std::ops::Range<usize>,
    connections: Vec<Connection>,
}
impl ProgramInputPorts {
    pub(in crate::domain_computation::primary_graph) fn append(
        features: &mut Vec<Feature>,
        programs: &mut Vec<Self>,
        original_features: &[Feature],
        original_connections: &[Connection],
    ) {
        let start = features.len();
        features.extend_from_slice(original_features);
        programs.push(Self {
            features: start..features.len(),
            connections: original_connections.to_vec(),
        });
    }
}

pub(in crate::domain_computation::primary_graph::application_installation) fn validate<
    S: ApplicationSchema,
>(
    producers: &PendingProducerRegistry<S>,
    support: Option<&WorthQueryAdmittedProgramSupport<S>>,
) -> Result<(), Denial> {
    let refuse = |subject: &str| Denial::new(Kind::ProducerBindingMeaningMismatch, subject);
    let Some(support) = support else {
        return match producers.input_capture_contracts().next() {
            Some((producer, _, _)) => Err(refuse(producer)),
            None => Ok(()),
        };
    };
    for program in &support.input_programs {
        let features = &support.features[program.features.clone()];
        for connection in program
            .connections
            .iter()
            .filter(|c| c.input_capture_type().is_some())
        {
            let Some((_, family, capture)) =
                producers.input_capture_contracts().find(|(_, _, capture)| {
                    Some(capture.definition.into_external()) == connection.input_capture_type()
                })
            else {
                return Err(refuse(connection.identity()));
            };
            if connection.input_capture_identity() != Some(capture.identity)
                || connection.occurrence_component().is_none()
                || !features.iter().any(|feature| {
                    feature.identity() == connection.target_feature()
                        && feature.composition_instance() == connection.target_instance()
                        && feature
                            .derived_artifacts()
                            .iter()
                            .any(|artifact| artifact.producer_family() == family)
                })
                || !capture
                    .ports
                    .iter()
                    .zip(&capture.types)
                    .any(|(port, kind)| {
                        same_port(port, connection)
                            && Some(*kind) == connection.input_capture_port_type()
                    })
            {
                return Err(refuse(connection.identity()));
            }
        }
        for (producer, family, capture) in producers.input_capture_contracts() {
            if !features
                .iter()
                .flat_map(Feature::derived_artifacts)
                .any(|artifact| artifact.producer_family() == family)
            {
                continue;
            }
            let matching = program
                .connections
                .iter()
                .filter(|c| c.input_capture_type() == Some(capture.definition.into_external()))
                .collect::<Vec<_>>();
            if matching.is_empty() {
                return Err(refuse(producer));
            }
            for placement in &matching {
                for (port, kind) in capture.ports.iter().zip(&capture.types) {
                    if !matching.iter().any(|actual| {
                        actual.occurrence_component() == placement.occurrence_component()
                            && actual.input_capture_port_type() == Some(*kind)
                            && same_port(port, actual)
                    }) {
                        return Err(refuse(producer));
                    }
                }
            }
        }
    }
    Ok(())
}

fn same_port(expected: &Connection, actual: &Connection) -> bool {
    expected.identity() == actual.identity()
        && expected.source_instance() == actual.source_instance()
        && expected.source_feature() == actual.source_feature()
        && expected.source_port() == actual.source_port()
        && expected.target_instance() == actual.target_instance()
        && expected.target_feature() == actual.target_feature()
        && expected.target_port() == actual.target_port()
        && expected.target_required() == actual.target_required()
        && expected.exports_across_instances() == actual.exports_across_instances()
}
