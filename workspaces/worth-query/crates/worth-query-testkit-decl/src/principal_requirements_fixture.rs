//! Infallible principal requirements are checked against the composed schema.
pub use worth_query_decl::facade::{
    application_schema::{fragment::*, *},
    authentication::*,
};
pub struct Model<const DAMAGE: u8>;
pub struct Bindings<const DAMAGE: u8>;
pub struct Mapping<const DAMAGE: u8>;
pub struct Principal<const DAMAGE: u8>;
pub struct External<const DAMAGE: u8>;
pub struct Status<const DAMAGE: u8>;
pub struct Identity<const DAMAGE: u8>;
pub struct MapsTo<const DAMAGE: u8>;
pub struct ExternalRow(WorthQueryExternalPrincipalIdentity);
pub struct StatusRow(WorthQueryPrincipalMappingStatus);
pub struct IdentityRow {
    value: u64,
    nested: NestedIdentityRow,
}
pub struct NestedIdentityRow(u64);
pub struct Actor;
pub struct Foreign<const DAMAGE: u8>;
impl<const D: u8> Entity for Foreign<D> {
    const NAME: &'static str = "foreign_owner";
    type Aspects = (External<0>, Status<0>, Identity<0>);
    type Relations = ();
}

impl<const D: u8> Fragment for Model<D> {
    const NAME: &'static str = "principal_requirements";
    type Bindings = Bindings<D>;
    type Members = (Mapping<D>, Principal<D>, Foreign<D>);
}
impl<const D: u8> ApplicationSchema for Model<D> {
    const NAME: &'static str = "principal_requirements";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration_builder(Self)?
            .principal_binding(reference::<D>())
            .build()
    }
}
impl<const D: u8> Entity for Mapping<D> {
    const NAME: &'static str = "mapping";
    type Aspects = (External<D>, Status<D>);
    type Relations = (MapsTo<D>,);
}
impl<const D: u8> Entity for Principal<D> {
    const NAME: &'static str = "principal";
    type Aspects = (Identity<D>,);
    type Relations = ();
}
impl<const D: u8> Aspect for External<D> {
    const NAME: &'static str = if D == 2 { "other_external" } else { "external" };
    type Row = ExternalRow;
}
impl<const D: u8> Aspect for Status<D> {
    const NAME: &'static str = if D == 8 { "other_status" } else { "status" };
    type Row = StatusRow;
}
impl<const D: u8> Aspect for Identity<D> {
    const NAME: &'static str = if D == 9 { "other_identity" } else { "identity" };
    type Row = IdentityRow;
}
impl<const D: u8> Relation for MapsTo<D> {
    const NAME: &'static str = "maps_to";
    type To = Principal<D>;
}
impl<const D: u8> BindingSet for Bindings<D> {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl<const D: u8> Scalar<WorthQueryExternalPrincipalIdentity> for Bindings<D> {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = D == 3;
    const EQUALITY_QUERYABLE: bool = true;
}
impl<const D: u8> Scalar<WorthQueryPrincipalMappingStatus> for Bindings<D> {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = D != 4;
}
impl<const D: u8> Scalar<u64> for Bindings<D> {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = D == 5;
    const EQUALITY_QUERYABLE: bool = D != 6;
}
impl<const D: u8> Record<ExternalRow> for Bindings<D> {
    fn record() -> impl Shape<ExternalRow, Self> {
        field("value", |r: &ExternalRow| r.0.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalRow(value))
            },
        )
    }
}
impl<const D: u8> Record<StatusRow> for Bindings<D> {
    fn record() -> impl Shape<StatusRow, Self> {
        field("value", |r: &StatusRow| r.0).build(|value, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(StatusRow(value))
        })
    }
}
impl<const D: u8> Record<IdentityRow> for Bindings<D> {
    fn record() -> impl Shape<IdentityRow, Self> {
        field("value", |r: &IdentityRow| r.value)
            .and("nested", |r: &IdentityRow| NestedIdentityRow(r.nested.0))
            .build(|value, nested, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityRow { value, nested })
            })
    }
}
pub type Reference<const D: u8> = ApplicationPrincipalBindingRef<
    Model<D>,
    Actor,
    Mapping<0>,
    Principal<0>,
    u64,
    U64ApplicationValueBinding,
>;
pub fn reference<const D: u8>() -> Reference<D> {
    ApplicationPrincipalBindingRef::from_requirements(
        "actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                record_field(
                    Model::<0>,
                    Mapping::<0>,
                    External::<0>,
                    if D == 1 {
                        "typo"
                    } else if D == 11 {
                        "bad field"
                    } else {
                        "value"
                    },
                ),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                record_field(
                    Model::<0>,
                    Mapping::<0>,
                    Status::<0>,
                    if D == 7 { "typo" } else { "value" },
                ),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(ApplicationRelationRef::<
                Model<D>,
                MapsTo<0>,
                Mapping<0>,
                Principal<0>,
            >::from_schema_identifiers(
                "maps_to",
                "mapping",
                "principal",
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            )),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                Model::<0>,
                record_field(
                    Model::<0>,
                    Principal::<0>,
                    Identity::<0>,
                    if D == 10 {
                        "typo"
                    } else if D == 12 {
                        "nested.value"
                    } else {
                        "value"
                    },
                ),
            ),
        },
    )
}

impl<const D: u8> Scalar<String> for Bindings<D> {
    type Binding = StringApplicationValueBinding;
}
/// A valid path and posture with the wrong selected scalar codec.
pub fn wrong_codec_reference() -> ApplicationPrincipalBindingRef<
    Model<0>,
    Actor,
    Mapping<0>,
    Principal<0>,
    String,
    StringApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                record_field(Model::<0>, Mapping::<0>, External::<0>, "value"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                record_field(Model::<0>, Mapping::<0>, Status::<0>, "value"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                Model::<0>,
                Mapping::<0>,
                MapsTo::<0>,
            )),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                Model::<0>,
                record_field(Model::<0>, Principal::<0>, Identity::<0>, "value"),
            ),
        },
    )
}

impl<const D: u8> Record<NestedIdentityRow> for Bindings<D> {
    fn record() -> impl Shape<NestedIdentityRow, Self> {
        field("value", |row: &NestedIdentityRow| row.0).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NestedIdentityRow(value))
            },
        )
    }
}
