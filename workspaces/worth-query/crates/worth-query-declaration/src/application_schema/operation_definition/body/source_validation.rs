//! Cold source closure keeps scalar copying and relation selection at their declared scope.
use crate::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyCreatedField as Step,
    ApplicationOperationDecisionReadTarget as Read, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};

pub(super) fn validate(
    operation: &str,
    scope: &str,
    step: &Step,
    members: &[Member],
) -> Result<(), Denial> {
    let Some(source) = step.source_field() else {
        return Ok(());
    };
    let target = step.field();
    let target_metadata = members.iter().find_map(|member| match member {
        Member::Field {
            entity,
            aspect,
            field,
            unit,
            presence,
            ..
        } if entity == target.entity() && aspect == target.aspect() && field == target.field() => {
            Some((unit, presence))
        }
        _ => None,
    });
    let related_closed = if let Some(path) = step.path_source() {
        path.path.start_entity() == scope && path.has_closed_reads(operation, members)
    } else {
        step.related_source().map_or_else(
            || source.entity() == scope,
            |related| {
                related.from == scope
                    && related.to == source.entity()
                    && members.iter().any(|member| {
                        matches!(member, Member::Relation { relation, from, to, .. }
                if relation == &related.relation && from == &related.from && to == &related.to)
                    })
                    && related
                        .decision_reads()
                        .iter()
                        .all(|read| has_read(members, operation, read))
                    && target_metadata.is_some_and(|(_, presence)| *presence == related.presence)
            },
        )
    };
    if !related_closed || target_metadata.is_none()
        || !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, scalar_family, value_type, unit, presence, .. }
            if entity == source.entity() && aspect == source.aspect() && field == source.field()
                && value_type == step.binding().as_str() && *scalar_family == step.scalar_family()
                && Some(*presence) == step.source_presence() && target_metadata.map(|(target_unit, _)| target_unit) == Some(unit)))
        || !step.source_read().is_some_and(|read| has_read(members, operation, &read))
    {
        return Err(Denial::MissingOperationDecisionReadDependency);
    }
    if matches!(
        step,
        Step::PresentScopeField { .. } | Step::CapturedScopeField { .. }
    ) && !target_metadata.is_some_and(|(_, presence)| *presence == Presence::Required)
    {
        return Err(Denial::MissingOperationDecisionReadDependency);
    }
    if let Step::CapturedPathField {
        target_presence, ..
    } = step
        && !target_metadata.is_some_and(|(_, presence)| presence == target_presence)
    {
        return Err(Denial::MissingOperationDecisionReadDependency);
    }
    Ok(())
}
fn has_read(members: &[Member], operation: &str, read: &Read) -> bool {
    members.iter().any(|member| matches!(member, Member::OperationDecisionRead { operation: candidate, target } if candidate == operation && target == read))
}
