use super::*;

mod query_definition;
use query_definition::query_definition;

pub(super) struct Schema;
struct FirstInput;
struct SecondInput;
struct BlankInput;
struct FirstOperation;
struct SecondOperation;
struct BlankOperation;
struct FirstPayload;
struct SecondPayload;
struct FirstEffect;
struct SecondEffect;
struct QueryEntity;
struct QueryParameters;

impl fragment::Entity for QueryEntity {
    type Aspects = ();
    type Relations = ();
    const NAME: &'static str = "QueryEntity";
}

struct PortableEntity;
impl fragment::Entity for PortableEntity {
    const NAME: &'static str = "PortableEntity";
    type Aspects = ();
    type Relations = ();
}
struct PortableRecords;
impl fragment::Fragment for PortableRecords {
    const NAME: &'static str = "portable";
    type Bindings = fragment::NoBindings;
    type Members = (PortableEntity,);
}

crate::worth_query_capability_context!(
    FirstContext in Schema,
    identity "worth.query.test.colliding-context.v1"
);
crate::worth_query_capability_context!(
    SecondContext in Schema,
    identity "worth.query.test.colliding-context.v1"
);
crate::worth_query_capability_context!(BlankContext in Schema, identity "");
crate::worth_query_capability_context_entity_slot!(
    FirstSlot in Schema,
    FirstContext => PortableEntity,
    identity "worth.query.test.colliding-slot.v1"
);
crate::worth_query_capability_context_entity_slot!(
    BlankSlot in Schema,
    FirstContext => PortableEntity,
    identity ""
);
crate::worth_query_capability_context_entity_slot!(
    SecondSlot in Schema,
    FirstContext => PortableEntity,
    identity "worth.query.test.colliding-slot.v1"
);
crate::worth_query_capability_provenance!(
    FirstProvenance in Schema,
    identity "worth.query.test.colliding-provenance.v1"
);
crate::worth_query_capability_provenance!(BlankProvenance in Schema, identity "");
crate::worth_query_capability_provenance!(
    SecondProvenance in Schema,
    identity "worth.query.test.colliding-provenance.v1"
);

crate::worth_query_structured_value_binding!(FirstQueryParametersBinding for QueryParameters { identity: "worth.query.test.query-parameters.v1" });
crate::worth_query_structured_value_binding!(FirstQueryResultBinding for () { identity: "worth.rust.unit" });
crate::worth_query_application_query!(
    FirstQuery for Schema,
    identity "worth.query.test.colliding-query.v1",
    parameters FirstQueryParametersBinding,
    result FirstQueryResultBinding,
    scope QueryEntity => "worth.query.test.query-scope.v1",
    name "first_query"
);
crate::worth_query_structured_value_binding!(SecondQueryParametersBinding for QueryParameters { identity: "worth.query.test.query-parameters.v1" });
crate::worth_query_structured_value_binding!(SecondQueryResultBinding for () { identity: "worth.rust.unit" });
crate::worth_query_application_query!(
    SecondQuery for Schema,
    identity "worth.query.test.colliding-query.v1",
    parameters SecondQueryParametersBinding,
    result SecondQueryResultBinding,
    scope QueryEntity => "worth.query.test.query-scope.v1",
    name "second_query"
);
crate::worth_query_structured_value_binding!(BlankQueryParametersBinding for QueryParameters { identity: "worth.query.test.query-parameters.v1" });
crate::worth_query_structured_value_binding!(BlankQueryResultBinding for () { identity: "worth.rust.unit" });
crate::worth_query_application_query!(
    BlankQuery for Schema,
    identity "",
    parameters BlankQueryParametersBinding,
    result BlankQueryResultBinding,
    scope QueryEntity => "worth.query.test.query-scope.v1",
    name "blank_query"
);

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "WORTH.tests";
    const NAME: &'static str = "portable-member-identity";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema().build()
    }
}

worth_query_portable_type!(FirstInput => "worth.query.test.first-input.v1");
worth_query_portable_type!(SecondInput => "worth.query.test.second-input.v1");
worth_query_portable_type!(BlankInput => "");
worth_query_portable_type!(FirstPayload => "worth.query.test.first-payload.v1");
worth_query_portable_type!(SecondPayload => "worth.query.test.second-payload.v1");

macro_rules! operation_identity {
    ($operation:ty, $binding:ident, $input:ty, $input_identity:literal, $identifier:literal) => {
        crate::worth_query_structured_value_binding!(
            $binding for $input { identity: $input_identity }
        );
        impl ApplicationOperationMarkerIdentity<Schema> for $operation {
            type InputBinding = $binding;
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

operation_identity!(
    FirstOperation,
    FirstInputBinding,
    FirstInput,
    "worth.query.test.first-input.v1",
    "SameOperation"
);
operation_identity!(
    SecondOperation,
    SecondInputBinding,
    SecondInput,
    "worth.query.test.second-input.v1",
    "SameOperation"
);
operation_identity!(
    BlankOperation,
    BlankInputBinding,
    BlankInput,
    "",
    "BlankOperation"
);

macro_rules! effect_identity {
    ($effect:ty, $binding:ident, $payload:ty, $payload_identity:literal) => {
        crate::worth_query_structured_value_binding!(
            $binding for $payload { identity: $payload_identity }
        );
        impl ApplicationEffectMarkerIdentity<Schema> for $effect {
            type PayloadBinding = $binding;
            const IDENTIFIER: &'static str = "SameEffect";
        }

    };
}

effect_identity!(
    FirstEffect,
    FirstPayloadBinding,
    FirstPayload,
    "worth.query.test.first-payload.v1"
);
effect_identity!(
    SecondEffect,
    SecondPayloadBinding,
    SecondPayload,
    "worth.query.test.second-payload.v1"
);

#[test]
fn one_operation_name_cannot_redeclare_a_different_input_identity() {
    let first = ApplicationOperationRef::<Schema, FirstOperation, FirstInput>::from_declaration();
    let second =
        ApplicationOperationRef::<Schema, SecondOperation, SecondInput>::from_declaration();
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(
            first
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation(
            second
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
        .unwrap_err();
    assert_eq!(denial, ApplicationSchemaDeclarationDenial::DuplicateMember);
}

#[test]
fn one_effect_name_cannot_redeclare_a_different_payload_identity() {
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .effect(ApplicationEffectRef::<Schema, FirstEffect, FirstPayload>::from_declaration())
        .effect(ApplicationEffectRef::<Schema, SecondEffect, SecondPayload>::from_declaration())
        .build()
        .unwrap_err();
    assert_eq!(denial, ApplicationSchemaDeclarationDenial::DuplicateMember);
}

#[test]
fn blank_portable_type_identity_is_rejected_before_canonical_meaning() {
    let operation =
        ApplicationOperationRef::<Schema, BlankOperation, BlankInput>::from_declaration();
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );
}

#[test]
fn two_query_markers_cannot_claim_one_portable_query_identity() {
    let entity = ApplicationEntityRef::<Schema, QueryEntity>::from_schema_identifier("QueryEntity");
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(QueryRecords)
        .unwrap()
        .application_query(query_definition(FirstQuery::reference(), entity))
        .application_query(query_definition(SecondQuery::reference(), entity))
        .build()
        .unwrap_err();
    assert_eq!(denial, ApplicationSchemaDeclarationDenial::DuplicateMember);
}

#[test]
fn blank_query_identity_is_rejected_before_schema_canonicalization() {
    let entity = ApplicationEntityRef::<Schema, QueryEntity>::from_schema_identifier("QueryEntity");
    let denial = match query_definition(BlankQuery::reference(), entity).build() {
        Ok(_) => panic!("blank query identity must be rejected"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        crate::application_query::ApplicationQueryDefinitionDenial::InvalidPortableIdentity
    );
}

#[test]
fn capability_marker_identity_collisions_are_rejected() {
    let context_denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .capability_context(FirstContext::reference())
        .capability_context(SecondContext::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        context_denial,
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );

    let slot_denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(PortableRecords)
        .unwrap()
        .capability_context(FirstContext::reference())
        .capability_context_entity_slot(FirstSlot::reference())
        .capability_context_entity_slot(SecondSlot::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        slot_denial,
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );

    let provenance_denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .capability_provenance(FirstProvenance::reference())
        .capability_provenance(SecondProvenance::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        provenance_denial,
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}

#[test]
fn blank_capability_marker_identity_is_rejected() {
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .capability_context(BlankContext::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );

    let slot_denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(PortableRecords)
        .unwrap()
        .capability_context(FirstContext::reference())
        .capability_context_entity_slot(BlankSlot::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        slot_denial,
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );

    let provenance_denial = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .capability_provenance(BlankProvenance::reference())
        .build()
        .unwrap_err();
    assert_eq!(
        provenance_denial,
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );
}

mod module_moves;

struct QueryRecords;
impl fragment::Fragment for QueryRecords {
    const NAME: &'static str = "query_records";
    type Bindings = fragment::NoBindings;
    type Members = (QueryEntity,);
}
