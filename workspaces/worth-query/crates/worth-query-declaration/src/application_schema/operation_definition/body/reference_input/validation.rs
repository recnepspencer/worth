//! Fresh admission closes lookup identity, nullable cardinality and all read/link/unlink contracts.
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyAuthoringDenial as Authoring,
    ApplicationOperationBodyProgram as Body, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
pub(in crate::application_schema::operation_definition::body) fn validate_shape(
    body: &Body,
) -> Result<(), Authoring> {
    let mut relations = std::collections::BTreeSet::new();
    for reference in body.reference_inputs() {
        if Some(reference.from.as_str()) != body.scope_entity()
            || reference.to != reference.field.entity()
            || [
                &reference.relation,
                &reference.from,
                &reference.to,
                &reference.member,
                &reference.key,
            ]
            .iter()
            .any(|part| part.is_empty() || part.trim() != part.as_str())
            || !relations.insert(&reference.relation)
            || body
                .creation()
                .into_iter()
                .flat_map(|root| root.initialization_sites())
                .flat_map(|site| {
                    site.links()
                        .iter()
                        .chain(site.path_links().iter().map(|p| &p.link))
                })
                .any(|link| link.relation == reference.relation)
            || body.retirements().iter().any(|retirement| {
                retirement.owners_relation == reference.relation
                    || retirement.selection_relation == reference.relation
            })
        {
            return Err(Authoring::InvalidReferenceReplacement);
        }
    }
    Ok(())
}
pub(in crate::application_schema::operation_definition::body) fn validate(
    operation: &str,
    body: &Body,
    members: &[Member],
) -> Result<(), Denial> {
    validate_shape(body).map_err(|_| Denial::program_body_dependency(operation))?;
    for reference in body.reference_inputs() {
        if !members.iter().any(|member|matches!(member,Member::Relation{relation,from,to,integrity} if relation==&reference.relation && from==&reference.from && to==&reference.to && integrity.cardinality.source_max==Some(1) && integrity.cardinality.source_min.is_none_or(|minimum|minimum==0)))
            || !members.iter().any(|member|matches!(member,Member::Field{entity,aspect,field,value_type,scalar_family,presence:ApplicationFieldPresence::Required,writable:false,equality_queryable:true,unique:true,unit:None,frame:None,..} if entity==reference.field.entity() && aspect==reference.field.aspect() && field==reference.field.field() && value_type==reference.binding.as_str() && scalar_family==&reference.scalar_family)) {
            return Err(Denial::program_body_dependency(operation));
        }
        for read in reference.decision_reads() {
            if !members.iter().any(|member|matches!(member,Member::OperationDecisionRead{operation:declared,target} if declared==operation && target==&read)) {return Err(Denial::decision_read_body_dependency(operation));}
        }
        for target in reference.program_targets() {
            if !members.iter().any(|member|matches!(member,Member::OperationProgram{operation:declared,target:declared_target} if declared==operation && declared_target==&target)) {return Err(Denial::program_body_dependency(operation));}
        }
    }
    Ok(())
}
