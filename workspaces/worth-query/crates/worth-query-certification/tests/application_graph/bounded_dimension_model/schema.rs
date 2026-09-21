//! The authored meaning both bounded-dimension programs are written against.
//!
//! One schema installs one part entity carrying a numeric dimension, one
//! operation that sets it, one ordinary read, and BOTH bounded-dimension rule
//! contracts. Which of the two actually decides a candidate is not a property
//! of this schema: it is decided by the program the occurrence is running.

use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    worth_query_application, worth_query_application_contribution, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding,
    worth_query_relation, worth_query_structured_value_binding,
};

#[path = "schema/bounded_dimension_invariants.rs"]
mod bounded_dimension_invariants;
#[path = "schema/part_dimension_query.rs"]
mod part_dimension_query;

pub use bounded_dimension_invariants::{
    BoundedDimensionV1, BoundedDimensionV2, BoundedDimensionV3,
};
pub use part_dimension_query::{
    part_dimension_condition_query_definition, part_dimension_query_definition,
    PartDimensionConditionBinding, PartDimensionConditionQuery, PartDimensionQuery,
    PartDimensionRow, PartDimensionRowBinding, PartQueryParametersBinding,
};

worth_query_application! {
    pub BoundedDimensionSchema {
        owner: "bounded_dimension_courtroom",
        version: (1, 0),
        contributions: [BoundedDimensionContribution],
    }
}

worth_query_application_contribution! {
    pub contribution BoundedDimensionContribution in BoundedDimensionSchema {
        identity: "bounded_dimension_courtroom.application_graph.v1",
        members: |schema| {
            let schema = schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Part::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(Part::reference(), PartFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Part::reference(), PartIdentityField::reference())
                .field(Part::reference(), PartDimensionField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(PartPrincipalBinding::reference())
                .operation(
                    SetPartDimension::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_decision_fact_budget(SetPartDimension::reference(), 4)
                .operation_projection_work_budget(SetPartDimension::reference(), 8)
                .operation_read_field(SetPartDimension::reference(), PartIdentityField::reference())
                .operation_read_field(SetPartDimension::reference(), PartDimensionField::reference())
                .operation_write(SetPartDimension::reference(), PartDimensionField::reference())
                .invariant(bounded_dimension_invariants::first_definition())
                .invariant(bounded_dimension_invariants::second_definition())
                .application_query(part_dimension_query_definition())
                .application_query(part_dimension_condition_query_definition());
            super::workflow::declare(super::assessment_output::declare(super::dimension_entry::declare(schema)))
        }
    }
}

worth_query_entity!(pub ExternalMapping for BoundedDimensionSchema);
worth_query_entity!(pub Principal for BoundedDimensionSchema);
worth_query_entity!(pub Part for BoundedDimensionSchema);
worth_query_aspect!(pub ExternalIdentity for BoundedDimensionSchema, ExternalMapping; identity = AspectIdentity(0x91750101), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for BoundedDimensionSchema, Principal; identity = AspectIdentity(0x91750102), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PartFacts for BoundedDimensionSchema, Part; identity = AspectIdentity(0x91750103), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for BoundedDimensionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity => declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for BoundedDimensionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus => declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for BoundedDimensionSchema, Principal, PrincipalFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub PartIdentityField for BoundedDimensionSchema, Part, PartFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub PartDimensionField for BoundedDimensionSchema, Part, PartFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in BoundedDimensionSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub PartPrincipalBinding in BoundedDimensionSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

/// The exact dimension one request asks a named part to carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetPartDimensionInput {
    pub identity: String,
    pub dimension: u64,
}
worth_query_portable_type!(SetPartDimensionInput => "worth.query.certification.bounded-dimension.set-input.v1");
worth_query_structured_value_binding!(pub SetPartDimensionInputBinding for SetPartDimensionInput {
    identity: "worth.query.certification.bounded-dimension.set-input.v1"
});
worth_query_operation!(pub SetPartDimension for BoundedDimensionSchema, input SetPartDimensionInputBinding);
worth_query_operation_reads!(SetPartDimension => [PartIdentityField, PartDimensionField]);
worth_query_operation_writes!(SetPartDimension => [PartDimensionField]);

impl primary_graph::WorthQueryApplicationProjection<BoundedDimensionSchema, PartDimensionQuery>
    for PartDimensionRow
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            BoundedDimensionSchema,
            PartDimensionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(part_dimension_query::identity_result())?,
            dimension: row.field(part_dimension_query::dimension_result())?,
        })
    }
}

impl
    primary_graph::WorthQueryApplicationProjection<
        BoundedDimensionSchema,
        PartDimensionConditionQuery,
    > for bool
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            BoundedDimensionSchema,
            PartDimensionConditionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(row.field(part_dimension_query::condition_dimension_result())? > 0)
    }
}
