//! Compile exact atomic targets to installed physical read descriptions.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn compile_reads(
    schema: &str,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    reads: Vec<ApplicationOperationDecisionReadTarget>,
) -> SpineResult<Vec<WorthQueryPortableOperationGraphReadScope>> {
    let mut entities = BTreeSet::new();
    let mut projections = BTreeMap::<(String, String), BTreeSet<String>>::new();
    let mut relations = BTreeSet::new();
    for read in reads {
        match read {
            // The Parent and its Source admit their own reads. The operation's
            // physical projection declarations are checked independently.
            ApplicationOperationDecisionReadTarget::ManagedIndex { .. } => {}
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
        let mask = projection_mask::compile(native, &fields)?;
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
