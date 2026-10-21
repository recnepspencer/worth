//! Fresh retirement admission reconstructs all trigger, ownership and effect correspondence.
use super::super::ApplicationOperationBodyProgram;
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
pub(in crate::application_schema::operation_definition::body) fn validate(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    let mut paths = std::collections::BTreeSet::new();
    for rule in body.retirements() {
        if !rule.fits_body(body) || !paths.insert((&rule.owners_relation,&rule.selection_relation))
            || !members.iter().any(|member|matches!(member,Member::Relation{relation,from,to,integrity,..}
                if relation==&rule.owners_relation && from==&rule.scope && to==&rule.owner && integrity.cardinality.target_max==Some(1)))
            || !members.iter().any(|member|matches!(member,Member::Relation{relation,from,to,integrity,..}
                if relation==&rule.selection_relation && from==&rule.owner && to==&rule.selected && integrity.cardinality.source_max==Some(1) && integrity.cardinality.target_max==Some(1)))
            || !rule.program_targets().iter().all(|expected|members.iter().any(|member|matches!(member,Member::OperationProgram{operation:declared,target} if declared==operation && target==expected))) {
            return Err(Denial::program_body_dependency(operation));
        }
        if !rule.decision_reads().iter().all(|expected|members.iter().any(|member|matches!(member,Member::OperationDecisionRead{operation:declared,target} if declared==operation && target==expected))) {
            return Err(Denial::decision_read_body_dependency(operation));
        }
    }
    Ok(())
}
