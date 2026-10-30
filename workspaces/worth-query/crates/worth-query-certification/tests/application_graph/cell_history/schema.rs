//! Cell identity and feature ownership; Native owns publication versions.
use worth_query_decl::facade::{application_schema::*, authentication::*, *};

worth_query_application! { pub Schema {
    owner: "worth.query.cell-history", version: (1, 0), contributions: [Contribution],
}}
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "cell-history.contribution.v1",
    members: |schema| {
        super::query::declare(super::lifecycle::declare(super::edit::declare(schema
            .entity(ExternalMapping::reference()).entity(Principal::reference())
            .entity(Feature::reference()).entity(Cell::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Feature::reference(), FeatureFacts::reference())
            .aspect(Cell::reference(), CellFacts::reference())
            .field(ExternalMapping::reference(), ExternalIdentityField::reference())
            .field(ExternalMapping::reference(), MappingStatus::reference())
            .field(Principal::reference(), PrincipalIdentity::reference())
            .field(Feature::reference(), FeatureKey::reference())
            .field(Cell::reference(), CellKey::reference())
            .field(Cell::reference(), CellValue::reference())
            .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
            .relation(OwnsCell::reference(), Feature::reference(), Cell::reference())
            .principal_binding(PrincipalBinding::reference())
        )))
    }
}}
worth_query_entity!(pub ExternalMapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Feature for Schema);
worth_query_entity!(pub Cell for Schema);
worth_query_aspect!(pub ExternalIdentity for Schema, ExternalMapping; identity=AspectIdentity(0x91811601), revision=AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity=AspectIdentity(0x91811602), revision=AspectContractRevision(1));
worth_query_aspect!(pub FeatureFacts for Schema, Feature; identity=AspectIdentity(0x91811603), revision=AspectContractRevision(1));
worth_query_aspect!(pub CellFacts for Schema, Cell; identity=AspectIdentity(0x91811604), revision=AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for Schema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for Schema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub FeatureKey for Schema, Feature, FeatureFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub CellKey for Schema, Cell, CellFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub CellValue for Schema, Cell, CellFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, ExternalMapping => Principal; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub OwnsCell in Schema, Feature => Cell; integrity=same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub PrincipalBinding in Schema, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatus, target: MappingTarget => Principal,
    principal_identity: PrincipalIdentity
});
