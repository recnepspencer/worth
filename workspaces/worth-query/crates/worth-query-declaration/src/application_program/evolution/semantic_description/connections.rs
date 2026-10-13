use super::*;

pub(super) fn append_connections(
    facts: &mut Vec<ApplicationSemanticFact>,
    connections: &[ApplicationConnectionDeclaration],
) {
    for connection in connections {
        if connection.is_demand_only_root() {
            facts.push(fact(
                ApplicationSemanticFamily::Connections,
                framed_parts(&[
                    connection.source_instance(),
                    connection.target_instance(),
                    connection.identity(),
                    "output-root",
                ]),
                framed_fields([("posture", "demand-only".to_owned())]),
            ));
        }
        facts.push(fact(
            ApplicationSemanticFamily::Connections,
            framed_parts(&[
                connection.source_instance(),
                connection.target_instance(),
                connection.identity(),
            ]),
            framed_fields([
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
            ]),
        ));
    }
}
