//! Minimal governed hidden-ordering schema fragment and entities used by
//! `retained_basis_cursor_disclosure_court`.

use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::*;

use super::retained_basis_cursor_disclosure_query::HiddenOrderContribution;

worth_query_principal_issuer!(pub HiddenIssuer);
worth_query_authentication_audience!(pub HiddenAudience);
worth_query_authentication_method!(pub HiddenMethod);
worth_query_disclosure_classification!(pub HiddenRankDisclosure);

#[derive(Clone, Copy)]
pub struct HiddenSchema;
impl Fragment for HiddenSchema {
    const NAME: &'static str = "hidden_order_vault";
    type Bindings = NoBindings;
    type Members = (HiddenOrderContribution,);
}
impl ApplicationSchema for HiddenSchema {
    const OWNER: &'static str = "worth.query.hidden-order-cursor";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for HiddenSchema {
    type Contributions = <Self as Fragment>::Members;
}

#[derive(Clone, Copy)]
pub struct VaultFrag;
impl Fragment for VaultFrag {
    const NAME: &'static str = "vault";
    type Bindings = VaultBindings;
    type Members = (ExtMap, Prin, Vault, Item, VaultGrant);
}
pub struct VaultBindings;
impl BindingSet for VaultBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for VaultBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for VaultBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for VaultBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<String> for VaultBindings {
    type Binding = StringApplicationValueBinding;
}

macro_rules! ent {
    ($n:ident, $a:ty, $r:ty) => {
        #[derive(Clone, Copy)]
        pub struct $n;
        impl Entity for $n {
            const NAME: &'static str = stringify!($n);
            type Aspects = $a;
            type Relations = $r;
        }
        impl $n {
            pub fn reference() -> ApplicationEntityRef<HiddenSchema, Self> {
                entity(VaultFrag, Self)
            }
        }
    };
}
ent!(ExtMap, (ExtId,), (MapTarget,));
ent!(Prin, (PrinFacts,), (GrGrantee, GrGrantor));
ent!(Vault, (VaultFacts,), (Holds,));
ent!(Item, (ItemFacts,), ());
ent!(VaultGrant, (GrantFacts,), (GrResource, GrParent));

pub struct ExtId;
pub type ExtRow = (
    WorthQueryExternalPrincipalIdentity,
    WorthQueryPrincipalMappingStatus,
);
impl Aspect for ExtId {
    const NAME: &'static str = "ExtId";
    type Row = ExtRow;
}
impl Record<ExtRow> for VaultBindings {
    fn record() -> impl Shape<ExtRow, Self> {
        field("ExtIdField", |r: &ExtRow| r.0.clone())
            .equality_queryable(true)
            .and("MapStatus", |r: &ExtRow| r.1)
            .equality_queryable(true)
            .build(|a, b, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>((a, b))
            })
    }
}
pub struct PrinFacts;
pub struct PrinRow(pub u64);
impl Aspect for PrinFacts {
    const NAME: &'static str = "PrinFacts";
    type Row = PrinRow;
}
impl Record<PrinRow> for VaultBindings {
    fn record() -> impl Shape<PrinRow, Self> {
        field("PrinId", |r: &PrinRow| r.0)
            .equality_queryable(true)
            .read_only()
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrinRow(a))
            })
    }
}
pub struct VaultFacts;
pub struct VaultRow(pub String);
impl Aspect for VaultFacts {
    const NAME: &'static str = "VaultFacts";
    type Row = VaultRow;
}
impl Record<VaultRow> for VaultBindings {
    fn record() -> impl Shape<VaultRow, Self> {
        field("VaultKey", |r: &VaultRow| r.0.clone())
            .equality_queryable(true)
            .read_only()
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(VaultRow(a))
            })
    }
}
pub struct ItemFacts;
pub struct ItemFactsRow(pub u64, pub String, pub String);
impl Aspect for ItemFacts {
    const NAME: &'static str = "ItemFacts";
    type Row = ItemFactsRow;
}
impl Record<ItemFactsRow> for VaultBindings {
    fn record() -> impl Shape<ItemFactsRow, Self> {
        field("ItemKey", |r: &ItemFactsRow| r.0)
            .equality_queryable(true)
            .read_only()
            .and("Label", |r: &ItemFactsRow| r.1.clone())
            .and("SecretRank", |r: &ItemFactsRow| r.2.clone())
            .build(|a, b, c, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ItemFactsRow(a, b, c))
            })
    }
}
pub struct GrantFacts;
pub type GrantRow = (String, u64, u64, u64, String, u64, u64, u64, u64);
impl Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantRow;
}
impl Record<GrantRow> for VaultBindings {
    fn record() -> impl Shape<GrantRow, Self> {
        field("GId", |r: &GrantRow| r.0.clone())
            .equality_queryable(true)
            .and("GAct", |r: &GrantRow| r.1)
            .equality_queryable(true)
            .read_only()
            .and("GPurp", |r: &GrantRow| r.2)
            .equality_queryable(true)
            .read_only()
            .and("GStat", |r: &GrantRow| r.3)
            .equality_queryable(true)
            .and("GScope", |r: &GrantRow| r.4.clone())
            .equality_queryable(true)
            .and("GNb", |r: &GrantRow| r.5)
            .equality_queryable(true)
            .read_only()
            .and("GNa", |r: &GrantRow| r.6)
            .equality_queryable(true)
            .read_only()
            .and("GDepth", |r: &GrantRow| r.7)
            .equality_queryable(true)
            .read_only()
            .and("GDisc", |r: &GrantRow| r.8)
            .equality_queryable(true)
            .read_only()
            .build(
                |a, b, c, d, e, f, g, h, i, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>((a, b, c, d, e, f, g, h, i))
                },
            )
    }
}

pub type ExtIdField = FieldSelection<VaultFrag, ExtMap, ExtId, WorthQueryExternalPrincipalIdentity>;
pub type MapStatus = FieldSelection<VaultFrag, ExtMap, ExtId, WorthQueryPrincipalMappingStatus>;
pub type PrinId = FieldSelection<VaultFrag, Prin, PrinFacts, u64>;
pub type VaultKey = FieldSelection<VaultFrag, Vault, VaultFacts, String>;
pub type ItemKey = FieldSelection<VaultFrag, Item, ItemFacts, u64>;
pub type Label = FieldSelection<VaultFrag, Item, ItemFacts, String>;
pub type SecretRank = FieldSelection<VaultFrag, Item, ItemFacts, String>;
pub type GId = FieldSelection<VaultFrag, VaultGrant, GrantFacts, String>;
pub type GAct = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GPurp = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GStat = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GScope = FieldSelection<VaultFrag, VaultGrant, GrantFacts, String>;
pub type GNb = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GNa = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GDepth = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;
pub type GDisc = FieldSelection<VaultFrag, VaultGrant, GrantFacts, u64>;

macro_rules! rel {
    ($n:ident, $f:ident -> $t:ident) => {
        pub struct $n;
        impl Relation for $n {
            const NAME: &'static str = stringify!($n);
            type To = $t;
        }
        impl $n {
            pub fn reference() -> ApplicationRelationRef<HiddenSchema, Self, $f, $t> {
                relation(VaultFrag, $f, Self)
            }
        }
    };
}
rel!(MapTarget, ExtMap -> Prin);
rel!(Holds, Vault -> Item);
rel!(GrGrantee, Prin -> VaultGrant);
rel!(GrGrantor, Prin -> VaultGrant);
rel!(GrResource, VaultGrant -> Vault);
rel!(GrParent, VaultGrant -> VaultGrant);

pub struct HiddenPrincipal;
impl HiddenPrincipal {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        HiddenSchema,
        Self,
        ExtMap,
        Prin,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "HiddenPrincipal",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExtIdField::at("ExtIdField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MapStatus::at("MapStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(MapTarget::reference()),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrinId::at("PrinId"),
                ),
            },
        )
    }
}
impl Registration<HiddenSchema> for HiddenPrincipal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HiddenSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<HiddenSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}
