//! Conditional private storage contracts preserve physical rows and installation bytes.
use super::endpoints;
use crate::aggregate_identity::projected_storage::{
    COORDINATE_ASPECT, COORDINATE_FIELDS, MEMBERSHIP_ASPECT, MEMBERSHIP_FIELDS,
};
use crate::aggregate_identity::record::{
    AGGREGATE_ASPECT, AGGREGATE_FIELDS, LINEAGE_ASPECT, LINEAGE_FIELDS,
};
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentityStorage, AspectBinding,
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaRegistryError,
    SchemaVersionId,
};
use worth_foundational::facade::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy, AspectIdentity, AspectKey,
    AspectMaskContract, FieldKey, ScalarAspectType,
};

/// Register the exact private grammar, including optional projected coordinates
/// only when an installation actually binds a projected aggregate source.
pub fn register_aggregate_identity_storage(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    storage: AggregateIdentityStorage,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let registry = registry.register_entity_kind(EntityKindRegistration {
        kind_id: storage.aggregate_kind,
        kind_name: "worth-aggregate-identity".into(),
        schema_id: schema.clone(),
        schema_version_id: version,
        aspect_contract_declarations: aggregate_declarations(storage),
    })?;
    registry.register_relation_kind(RelationKindRegistration {
        kind_id: storage.lineage_kind,
        kind_name: "worth-aggregate-lineage".into(),
        schema_id: schema.clone(),
        schema_version_id: version,
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: lineage_declarations(storage),
        relation_integrity: endpoints(
            "aggregate-lineage",
            storage.aggregate_kind,
            storage.aggregate_kind,
        ),
    })
}

pub(super) fn aggregate_declarations(
    storage: AggregateIdentityStorage,
) -> KindAspectContractDeclarations {
    let mut declarations = declaration(
        AGGREGATE_ASPECT,
        storage.aggregate_aspect,
        AGGREGATE_FIELDS,
        false,
        false,
    );
    if let Some(projected) = storage.projected {
        declarations.aspects.extend(
            declaration(
                COORDINATE_ASPECT,
                projected.coordinate_aspect,
                COORDINATE_FIELDS,
                false,
                true,
            )
            .aspects,
        );
    }
    declarations
}

pub(super) fn lineage_declarations(
    storage: AggregateIdentityStorage,
) -> KindAspectContractDeclarations {
    declaration(
        LINEAGE_ASPECT,
        storage.lineage_aspect,
        LINEAGE_FIELDS,
        true,
        false,
    )
}

pub(super) fn membership_declarations(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> KindAspectContractDeclarations {
    if super::member_domain(registry, contract).is_some_and(|domain| domain.entity_kind().is_none())
        && let Some(projected) = contract.storage.projected
    {
        return declaration(
            MEMBERSHIP_ASPECT,
            projected.membership_aspect,
            MEMBERSHIP_FIELDS,
            true,
            false,
        );
    }
    Default::default()
}

fn declaration(
    aspect: &str,
    identity: AspectIdentity,
    fields: &[(&str, ScalarAspectType)],
    relation: bool,
    optional: bool,
) -> KindAspectContractDeclarations {
    let &(first, scalar) = fields.first().expect("fixed aggregate grammar is nonempty");
    let mut shape = aspects().struct_fields().required(first, scalar);
    for &(field, kind) in &fields[1..] {
        shape = shape.required(field, kind);
    }
    let step = aspects()
        .contract()
        .for_key(AspectKey::new(aspect).expect("fixed aggregate aspect"))
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1));
    let shape = shape.finish().expect("fixed aggregate scalar grammar");
    let contract = if optional {
        step.struct_with(
            shape,
            AspectMaskContract::struct_fields(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
    } else {
        step.struct_aspect(shape)
    };
    KindAspectContractDeclarations::new(vec![DeclaredAspectContractBinding {
        binding: if relation {
            AspectBinding::RelationField {
                field: FieldKey::new(aspect).expect("fixed aggregate binding"),
            }
        } else {
            AspectBinding::EntityField {
                field: FieldKey::new(aspect).expect("fixed aggregate binding"),
            }
        },
        contract,
    }])
}
