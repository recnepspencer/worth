//! Schema-bound direct endpoint routes preserve relation multiplicity and field meaning.
use super::*;
use crate::derived_relation::DerivedRelationNode;

/// One direct result-relation route from a Source row to a projected endpoint.
/// Binding checks the actual schema-owned path; its name only joins compiled
/// projection slots and cannot admit any observation or child entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedIndexSourceProjectionRoute {
    name: String,
    pub(in crate::validation::candidate_source) path: DerivedRelationHandle,
    fields: Arc<[ManagedIndexSourceField]>,
}

impl ManagedIndexSourceProjectionRoute {
    pub fn new(
        name: String,
        path: DerivedRelationHandle,
        fields: Vec<ManagedIndexSourceField>,
    ) -> Result<Self, Denial> {
        if name.is_empty()
            || path.parts().from.entity_kind().is_none()
            || path.parts().to.entity_kind().is_none()
            || !matches!(
                path.parts().nodes.as_slice(),
                [DerivedRelationNode::Stored { .. }]
                    | [
                        DerivedRelationNode::Stored { .. },
                        DerivedRelationNode::Reverse { input: 0 }
                    ]
            )
        {
            return Err(Denial::InvalidExpression);
        }
        Ok(Self {
            name,
            path,
            fields: fields.into(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fields(&self) -> &[ManagedIndexSourceField] {
        &self.fields
    }
    pub fn endpoint_kind(&self) -> KindId {
        self.path
            .parts()
            .to
            .entity_kind()
            .expect("bound endpoint projection route")
    }

    pub fn physical_relation(&self) -> (KindId, bool) {
        match self.path.parts().nodes.as_slice() {
            [DerivedRelationNode::Stored { relation, .. }] => (*relation, true),
            [DerivedRelationNode::Stored { relation, .. }, DerivedRelationNode::Reverse { input: 0 }] => {
                (*relation, false)
            }
            _ => unreachable!("binding admitted one direct physical endpoint step"),
        }
    }

    pub(super) fn validate(
        &self,
        registry: &RelationalSchemaRegistry,
        row: KindId,
    ) -> Result<(), Denial> {
        if self.name.is_empty()
            || !registry.admits_derived_relation(&self.path)
            || self.path.parts().from.entity_kind() != Some(row)
            || !matches!(
                self.path.parts().nodes.as_slice(),
                [DerivedRelationNode::Stored { .. }]
                    | [
                        DerivedRelationNode::Stored { .. },
                        DerivedRelationNode::Reverse { input: 0 }
                    ]
            )
        {
            return Err(Denial::InvalidExpression);
        }
        let endpoint = self
            .path
            .parts()
            .to
            .entity_kind()
            .ok_or(Denial::InvalidMemberDomain)?;
        validate_fields(registry, endpoint, &self.fields)
    }
}

pub(super) fn validate_fields(
    registry: &RelationalSchemaRegistry,
    kind: KindId,
    fields: &[ManagedIndexSourceField],
) -> Result<(), Denial> {
    let registration = registry
        .entity_kinds
        .get(&kind)
        .ok_or(Denial::InvalidPhysicalSource(kind))?;
    for field in fields {
        if field.locator.field_path().fields().len() != 1
            || !registration.aspect_contract_declarations.aspects.iter().any(|aspect| {
                aspect.foundational_key() == field.locator.aspect().aspect_key()
                    && matches!(aspect.contract.shape(), worth_foundational::facade::AspectShape::Struct(shape)
                        if shape.field(&field.locator.field_path().fields()[0]).is_some())
            }) {
            return Err(Denial::InvalidExpression);
        }
    }
    Ok(())
}
