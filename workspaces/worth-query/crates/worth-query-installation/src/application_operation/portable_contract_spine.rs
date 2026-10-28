use std::collections::{BTreeMap, BTreeSet};
mod derived_reads;

use worth_foundational::facade::{aspects, CanonicalFieldPath, FieldKey};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableNativeAspectContractRecord, WorthQueryPortableOperationGraphReadScope,
    WorthQueryPortableOperationTouchScope,
};

use super::contract_resolution::{
    operation_decision_reads_from_members, operation_program_from_members,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryPortableOperationContractSpineDenialKind {
    MissingNativeAspect,
    MissingNativeField,
    InvalidProjectionMask,
    AmbiguousExternalEffect,
    AmbiguousAftermath,
    AmbiguousBody,
    InvalidDerivedSource,
}

type SpineResult<T> = Result<T, WorthQueryPortableOperationContractSpineDenialKind>;

pub(crate) fn compile_portable_operation_contract_records(
    schema: &ErasedApplicationSchemaDeclaration,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
) -> SpineResult<Vec<WorthQueryPortableApplicationOperationContractRecord>> {
    schema
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } => Some((operation, input_type)),
            _ => None,
        })
        .map(|(operation, input_type)| {
            compile_portable_operation_contract_record(
                schema.name(),
                schema.members(),
                native_contracts,
                operation,
                input_type.clone(),
            )
        })
        .collect()
}

pub(crate) fn compile_portable_operation_contract_record(
    schema: &str,
    members: &[ApplicationSchemaMember],
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    operation: &str,
    input_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
) -> SpineResult<WorthQueryPortableApplicationOperationContractRecord> {
    let mut reads = operation_decision_reads_from_members(members, operation, input_type.as_str());
    let derived_sources = derived_reads::append(members, operation, &mut reads)?;
    let mut graph_reads = compile_reads(schema, native_contracts, reads)?;
    graph_reads.extend(members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::OperationComponentRead { definition }
            if definition.operation() == operation =>
        {
            Some(WorthQueryPortableOperationGraphReadScope::Components {
                schema: schema.to_owned(),
                definition: definition.clone(),
            })
        }
        _ => None,
    }));
    graph_reads.extend(derived_sources.into_iter().map(|relation| {
        WorthQueryPortableOperationGraphReadScope::DerivedSource {
            schema: schema.to_owned(),
            relation,
        }
    }));
    let program = operation_program_from_members(members, operation, input_type.as_str());
    let touches = compile_touches(schema, native_contracts, &program)?;
    let mut emissions = program
        .iter()
        .filter_map(|target| match target {
            ApplicationOperationProgramTarget::Emit { effect } => Some(effect.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    emissions.sort();
    emissions.dedup();
    let external_effect = compile_external_effect(members, operation)?;
    let reconciliation = operation_reconciliation(members, operation)?;
    let mut bodies = members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::OperationBody {
            operation: candidate,
            body,
        } if candidate == operation => Some(body.clone()),
        _ => None,
    });
    let body = bodies.next();
    if bodies.next().is_some() {
        return Err(WorthQueryPortableOperationContractSpineDenialKind::AmbiguousBody);
    }
    Ok(WorthQueryPortableApplicationOperationContractRecord::new(
        schema.to_owned(),
        operation.to_owned(),
        input_type,
        graph_reads,
        touches,
        emissions,
        external_effect,
        reconciliation,
    )
    .with_body(body))
}

fn compile_reads(
    schema: &str,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    reads: Vec<ApplicationOperationDecisionReadTarget>,
) -> SpineResult<Vec<WorthQueryPortableOperationGraphReadScope>> {
    let mut entities = BTreeSet::new();
    let mut projections = BTreeMap::<(String, String), BTreeSet<String>>::new();
    let mut relations = BTreeSet::new();
    for read in reads {
        match read {
            ApplicationOperationDecisionReadTarget::Entity { entity } => {
                entities.insert(entity);
            }
            ApplicationOperationDecisionReadTarget::Field {
                entity,
                aspect,
                field,
            } => {
                projections
                    .entry((entity, aspect))
                    .or_default()
                    .insert(field);
            }
            ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
                relations.insert((relation, from, to));
            }
        }
    }
    let mut scopes = Vec::with_capacity(entities.len() + projections.len() + relations.len());
    scopes.extend(entities.into_iter().map(|entity| {
        WorthQueryPortableOperationGraphReadScope::Entity {
            schema: schema.to_owned(),
            entity,
        }
    }));
    for ((entity, aspect), fields) in projections {
        let native = native_contract(native_contracts, schema, &entity, &aspect)?;
        let fields = fields
            .into_iter()
            .map(|field| {
                native
                    .field(&field)
                    .cloned()
                    .ok_or(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeField)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mask = aspects()
            .projection_mask()
            .fields(fields.iter().map(FieldKey::as_str))
            .map_err(|_| {
                WorthQueryPortableOperationContractSpineDenialKind::InvalidProjectionMask
            })?;
        native
            .contract()
            .admits_projection_mask(&mask)
            .map_err(|_| {
                WorthQueryPortableOperationContractSpineDenialKind::InvalidProjectionMask
            })?;
        scopes.push(
            WorthQueryPortableOperationGraphReadScope::NativeProjection {
                schema: schema.to_owned(),
                entity,
                aspect: native.aspect().clone(),
                contract: native.contract().clone(),
                mask,
            },
        );
    }
    scopes.extend(relations.into_iter().map(|(relation, from, to)| {
        WorthQueryPortableOperationGraphReadScope::Relation {
            schema: schema.to_owned(),
            relation,
            from,
            to,
        }
    }));
    Ok(scopes)
}

fn compile_touches(
    schema: &str,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    program: &[ApplicationOperationProgramTarget],
) -> SpineResult<Vec<WorthQueryPortableOperationTouchScope>> {
    let mut touches = program
        .iter()
        .filter_map(|target| match target {
            ApplicationOperationProgramTarget::Emit { .. } => None,
            other => Some(compile_touch(schema, native_contracts, other)),
        })
        .collect::<SpineResult<Vec<_>>>()?;
    touches.sort_by(WorthQueryPortableOperationTouchScope::canonical_order);
    touches.dedup();
    Ok(touches)
}

fn compile_touch(
    schema: &str,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    target: &ApplicationOperationProgramTarget,
) -> SpineResult<WorthQueryPortableOperationTouchScope> {
    let schema = schema.to_owned();
    match target {
        ApplicationOperationProgramTarget::Create { entity } => {
            Ok(WorthQueryPortableOperationTouchScope::CreateEntity {
                schema,
                entity: entity.clone(),
            })
        }
        ApplicationOperationProgramTarget::Delete { entity } => {
            Ok(WorthQueryPortableOperationTouchScope::DeleteEntity {
                schema,
                entity: entity.clone(),
            })
        }
        ApplicationOperationProgramTarget::Write {
            entity,
            aspect,
            field,
        } => {
            let native = native_contract(native_contracts, &schema, entity, aspect)?;
            let field = native
                .field(field)
                .cloned()
                .ok_or(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeField)?;
            Ok(WorthQueryPortableOperationTouchScope::WriteField {
                schema,
                entity: entity.clone(),
                contract: native.contract().clone(),
                field_path: CanonicalFieldPath::single(field),
            })
        }
        ApplicationOperationProgramTarget::Link { relation, from, to } => {
            Ok(WorthQueryPortableOperationTouchScope::LinkRelation {
                schema,
                relation: relation.clone(),
                from: from.clone(),
                to: to.clone(),
            })
        }
        ApplicationOperationProgramTarget::InitializeRelationField {
            relation,
            from,
            to,
            aspect,
            field,
        } => {
            let native = native_relation_contract(native_contracts, &schema, relation, aspect)?;
            let field = native
                .field(field)
                .cloned()
                .ok_or(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeField)?;
            Ok(
                WorthQueryPortableOperationTouchScope::InitializeRelationField {
                    schema,
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    contract: native.contract().clone(),
                    field_path: CanonicalFieldPath::single(field),
                },
            )
        }
        ApplicationOperationProgramTarget::Unlink { relation, from, to } => {
            Ok(WorthQueryPortableOperationTouchScope::UnlinkRelation {
                schema,
                relation: relation.clone(),
                from: from.clone(),
                to: to.clone(),
            })
        }
        ApplicationOperationProgramTarget::Emit { .. } => unreachable!("emissions are separate"),
    }
}

fn native_contract<'a>(
    contracts: &'a [WorthQueryPortableNativeAspectContractRecord],
    schema: &str,
    entity: &str,
    aspect: &str,
) -> SpineResult<&'a WorthQueryPortableNativeAspectContractRecord> {
    contracts
        .iter()
        .find(|record| {
            record.schema() == schema
                && record.entity() == entity
                && record.aspect().as_str() == aspect
                && matches!(
                    record.binding(),
                    worth_foundational::facade::AspectBinding::EntityField { .. }
                )
        })
        .ok_or(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeAspect)
}

fn native_relation_contract<'a>(
    contracts: &'a [WorthQueryPortableNativeAspectContractRecord],
    schema: &str,
    relation: &str,
    aspect: &str,
) -> SpineResult<&'a WorthQueryPortableNativeAspectContractRecord> {
    contracts
        .iter()
        .find(|record| {
            record.schema() == schema
                && record.entity() == relation
                && record.aspect().as_str() == aspect
                && matches!(
                    record.binding(),
                    worth_foundational::facade::AspectBinding::RelationField { .. }
                )
        })
        .ok_or(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeAspect)
}

fn compile_external_effect(
    members: &[ApplicationSchemaMember],
    operation: &str,
) -> SpineResult<Option<WorthQueryPortableExternalEffectContractRecord>> {
    let mut declared = members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::OperationExternalEffect {
            operation: candidate,
            correlation_family,
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
        } if candidate == operation => Some(WorthQueryPortableExternalEffectContractRecord::new(
            correlation_family.clone(),
            effect.clone(),
            rust_payload_type.clone(),
            protocol.clone(),
            *maximum_payload_bytes,
        )),
        _ => None,
    });
    let first = declared.next();
    if declared.next().is_some() {
        return Err(WorthQueryPortableOperationContractSpineDenialKind::AmbiguousExternalEffect);
    }
    Ok(first)
}

fn operation_reconciliation(
    members: &[ApplicationSchemaMember],
    operation: &str,
) -> SpineResult<Option<WorthQueryPortableInstalledReconciliationProcedureRecord>> {
    let mut declared = members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::OperationAftermath {
            operation: candidate,
            contract,
        } if candidate == operation => contract.reconciliation().map(|procedure| {
            WorthQueryPortableInstalledReconciliationProcedureRecord::new(
                procedure.procedure_slot().to_owned(),
            )
        }),
        _ => None,
    });
    let first = declared.next();
    if declared.next().is_some() {
        return Err(WorthQueryPortableOperationContractSpineDenialKind::AmbiguousAftermath);
    }
    Ok(first)
}
