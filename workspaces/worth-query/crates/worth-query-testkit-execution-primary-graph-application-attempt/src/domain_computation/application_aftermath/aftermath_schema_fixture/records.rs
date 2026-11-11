//! One genuine row supplies every authorization and inverse selector.
use worth_query_declaration::facade::{
    application_schema::{
        fragment::{self, *},
        *,
    },
    authentication::*,
};
pub struct Fixture;
impl Fragment for Fixture {
    const NAME: &'static str = "aftermath";
    type Bindings = Bindings;
    type Members = (FixtureEntity,);
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<bool> for Bindings {
    type Binding = BoolApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<Balance> for Bindings {
    type Binding = BalanceBinding;
    const WRITABLE: bool = false;
}
#[derive(Clone, Copy)]
pub struct FixtureEntity;
impl Entity for FixtureEntity {
    const NAME: &'static str = "FixtureEntity";
    type Aspects = (IdentityAspect,);
    type Relations = (MappingTarget,);
}
pub struct IdentityAspect;
impl Aspect for IdentityAspect {
    const NAME: &'static str = "IdentityAspect";
    type Row = Facts;
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = FixtureEntity;
}
pub struct Facts {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
    principal: u64,
    frozen: bool,
    note: String,
    balance: Balance,
}
impl fragment::Record<Facts> for Bindings {
    fn record() -> impl Shape<Facts, Self> {
        field("ExternalIdentityField", |r: &Facts| r.identity.clone())
            .equality_queryable(true)
            .and("MappingStatusField", |r: &Facts| r.status)
            .and("PrincipalIdentityField", |r: &Facts| r.principal)
            .equality_queryable(true)
            .and("frozen", |r: &Facts| r.frozen)
            .and("note", |r: &Facts| r.note.clone())
            .and("balance", |r: &Facts| r.balance)
            .build(
                |identity,
                 status,
                 principal,
                 frozen,
                 note,
                 balance,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(Facts {
                        identity,
                        status,
                        principal,
                        frozen,
                        note,
                        balance,
                    })
                },
            )
    }
}
#[derive(Clone, Copy)]
pub struct Balance(u64);
impl Balance {
    fn get(&self) -> u64 {
        self.0
    }
    fn checked(value: u64) -> Option<Self> {
        Some(Self(value))
    }
}
worth_query_declaration::worth_query_value_binding!(pub BalanceBinding for Balance { identity: "worth.query.test.aftermath.balance.v1", scalar: UInt64, encode: Balance::get, decode: Balance::checked, });
pub type ExternalIdentityField =
    FieldSelection<Fixture, FixtureEntity, IdentityAspect, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatusField =
    FieldSelection<Fixture, FixtureEntity, IdentityAspect, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentityField = FieldSelection<Fixture, FixtureEntity, IdentityAspect, u64>;
pub type FrozenField = FieldSelection<Fixture, FixtureEntity, IdentityAspect, bool>;
pub type NoteField = FieldSelection<Fixture, FixtureEntity, IdentityAspect, String>;
pub type BalanceField = FieldSelection<Fixture, FixtureEntity, IdentityAspect, Balance>;
