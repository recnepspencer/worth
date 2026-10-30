pub use worth_query::facade::domain;

pub fn install_dependencies(
    dependencies: &[domain::WorthQuerySemanticTruthDependency],
    installed_signals: Vec<worth_signal::facade::InstalledSignalNodeCapability>,
    observation_record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    mapping_identity: &str,
    signal_partition: &str,
) -> Vec<domain::WorthQueryConditionalDependencyInstallation> {
    dependencies
        .iter()
        .zip(installed_signals)
        .enumerate()
        .map(|(ordinal, (dependency, installed_signal))| {
            let identity =
                crate::facade::certification::query_runtime_world::mapping_identity_for_dependency(
                    dependencies,
                    ordinal,
                    mapping_identity,
                );
            let target =
                worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
                    worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
                        identity,
                    ),
                    worth_signal::facade::PartitionToken::new(signal_partition),
                    installed_signal,
                );
            let source_record = source_record_for(dependency, observation_record);
            domain::WorthQueryConditionalDependencyInstallation::new(source_record, vec![target])
                .with_observation_record(observation_record)
        })
        .collect()
}

pub fn unique_aspect_contracts(
    dependencies: &[domain::WorthQuerySemanticTruthDependency],
) -> Vec<worth_foundational::facade::AspectContract> {
    let mut contracts = Vec::new();
    for dependency in dependencies {
        if !contracts.contains(dependency.contract()) {
            contracts.push(dependency.contract().clone());
        }
    }
    contracts
}

pub fn source_record_for(
    dependency: &domain::WorthQuerySemanticTruthDependency,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts> {
    if !matches!(
        dependency.locality(),
        domain::WorthQuerySemanticLocality::SourceRecord
    ) {
        return None;
    }
    Some(record)
}
