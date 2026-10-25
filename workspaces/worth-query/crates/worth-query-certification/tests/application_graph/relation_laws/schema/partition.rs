use super::*;

// compiled-example: begin
const EXPLICIT_CONTEXTS: ApplicationRelationIntegrity = ApplicationRelationIntegrity::new(
    ApplicationRelationEndpoints::new(true, ApplicationRelationCrossContextPolicy::AllowExplicit),
    ApplicationRelationCardinality::unbounded(),
    ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
);
const PARTITION_LOCAL: ApplicationRelationIntegrity = EXPLICIT_CONTEXTS.same_partition_endpoints();
// compiled-example: end

#[derive(Clone, Debug)]
pub struct CrossPartition;
impl LawPolicy for CrossPartition {
    const INTEGRITY: ApplicationRelationIntegrity = EXPLICIT_CONTEXTS;
}
#[derive(Clone, Debug)]
pub struct Isolated;
impl LawPolicy for Isolated {
    const INTEGRITY: ApplicationRelationIntegrity = PARTITION_LOCAL;
}
