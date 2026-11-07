//! Thirty-two distinct rows share one existing scalar codec and flat group.
use super::*;
use std::{convert::Infallible, marker::PhantomData};

macro_rules! each_case {
    ($apply:ident) => {
        $apply! {
            Case01: 1: "case01",
            Case02: 2: "case02",
            Case03: 3: "case03",
            Case04: 4: "case04",
            Case05: 5: "case05",
            Case06: 6: "case06",
            Case07: 7: "case07",
            Case08: 8: "case08",
            Case09: 9: "case09",
            Case10: 10: "case10",
            Case11: 11: "case11",
            Case12: 12: "case12",
            Case13: 13: "case13",
            Case14: 14: "case14",
            Case15: 15: "case15",
            Case16: 16: "case16",
            Case17: 17: "case17",
            Case18: 18: "case18",
            Case19: 19: "case19",
            Case20: 20: "case20",
            Case21: 21: "case21",
            Case22: 22: "case22",
            Case23: 23: "case23",
            Case24: 24: "case24",
            Case25: 25: "case25",
            Case26: 26: "case26",
            Case27: 27: "case27",
            Case28: 28: "case28",
            Case29: 29: "case29",
            Case30: 30: "case30",
            Case31: 31: "case31",
            Case32: 32: "case32"
        }
    };
}
pub(super) use each_case;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row<const N: u8>(pub u64);
#[derive(Debug)]
pub struct ZeroValue;
impl std::fmt::Display for ZeroValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("zero is outside this row's contract")
    }
}
impl std::error::Error for ZeroValue {}

macro_rules! define_cases {
    ($($case:ident: $n:literal: $name:literal),+) => {
        #[derive(Debug, Eq, PartialEq)]
        pub enum Value { $($case(Row<$n>),)+ }
        $(pub struct $case;
        impl Aspect for $case {
            const NAME: &'static str = $name;
            type Row = Row<$n>;
        }
        impl From<Row<$n>> for Value {
            fn from(row: Row<$n>) -> Self { Self::$case(row) }
        })+
    };
}
each_case!(define_cases);

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
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl<const N: u8> Record<Row<N>> for Bindings {
    fn record() -> impl Shape<Row<N>, Self> {
        field("number", |row: &Row<N>| row.0).build(|value, _: &mut ValueDecodeAdmission<'_>| {
            if value == 0 {
                Err(ZeroValue)
            } else {
                Ok(Row(value))
            }
        })
    }
}
pub struct IdentityRow(pub u64);
impl Record<IdentityRow> for Bindings {
    fn record() -> impl Shape<IdentityRow, Self> {
        field("id", |row: &IdentityRow| row.0)
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| Ok::<_, Infallible>(IdentityRow(id)))
    }
}
pub struct Identity;
impl Aspect for Identity {
    const NAME: &'static str = "identity";
    type Row = IdentityRow;
}

pub type OrderedCases = JoinCases<
    (
        Case01,
        Case02,
        Case03,
        Case04,
        Case05,
        Case06,
        Case07,
        Case08,
    ),
    JoinCases<
        (
            Case09,
            Case10,
            Case11,
            Case12,
            Case13,
            Case14,
            Case15,
            Case16,
        ),
        JoinCases<
            (
                Case17,
                Case18,
                Case19,
                Case20,
                Case21,
                Case22,
                Case23,
                Case24,
            ),
            (
                Case25,
                Case26,
                Case27,
                Case28,
                Case29,
                Case30,
                Case31,
                Case32,
            ),
        >,
    >,
>;
pub struct Form;
impl OneOf for Form {
    type Row = Value;
    type Cases = OrderedCases;
}
pub struct Reordered;
impl OneOf for Reordered {
    type Row = Value;
    type Cases = JoinCases<
        JoinCases<
            (
                Case32,
                Case31,
                Case30,
                Case29,
                Case28,
                Case27,
                Case26,
                Case25,
            ),
            (
                Case16,
                Case15,
                Case14,
                Case13,
                Case12,
                Case11,
                Case10,
                Case09,
            ),
        >,
        JoinCases<
            (
                Case24,
                Case23,
                Case22,
                Case21,
                Case20,
                Case19,
                Case18,
                Case17,
            ),
            (
                Case08,
                Case07,
                Case06,
                Case05,
                Case04,
                Case03,
                Case02,
                Case01,
            ),
        >,
    >;
}
pub struct Root<G>(PhantomData<G>);
impl<G: OneOf> Fragment for Root<G> {
    const NAME: &'static str = "large_cases";
    type Bindings = Bindings;
    type Members = (Entry<G>, Inspect<G>, super::principal::Authentication);
}
pub struct Entry<G>(PhantomData<G>);
impl<G: OneOf> Entity for Entry<G> {
    const NAME: &'static str = "entry";
    type Aspects = (Identity, Exclusive<G>);
    type Relations = ();
}
pub type Fixture = Root<Form>;
pub type Item = Entry<Form>;
pub const ROOT: Fixture = Root(PhantomData);
pub const ITEM: Item = Entry(PhantomData);
worth_query_host::facade::worth_query_structured_value_binding!(pub Input for () {
    identity: "large-cases.inspect-input.v1"
});
pub struct Inspect<G>(PhantomData<G>);
impl<G: OneOf> ApplicationOperationMarkerIdentity<Root<G>> for Inspect<G> {
    type InputBinding = Input;
    const IDENTIFIER: &'static str = "inspect";
}
impl<G: OneOf> OperationReads<Inspect<G>> for Entry<G> {}

macro_rules! schema {
    ($group:ident) => {
        impl Registration<Root<$group>> for Inspect<$group> {
            fn register(
                builder: ApplicationSchemaDeclarationBuilder<Root<$group>>,
            ) -> Result<
                ApplicationSchemaDeclarationBuilder<Root<$group>>,
                ApplicationSchemaDeclarationDenial,
            > {
                let operation =
                    ApplicationOperationRef::<Root<$group>, Self, ()>::from_declaration();
                Ok(builder
                    .operation(
                        operation
                            .definition()
                            .no_external_effect()
                            .no_aftermath()
                            .finish(),
                    )
                    .operation_read_entity(
                        operation,
                        entity(Root::<$group>(PhantomData), Entry::<$group>(PhantomData)),
                    )
                    .operation_read_record(
                        operation,
                        Root::<$group>(PhantomData),
                        Entry::<$group>(PhantomData),
                        Identity,
                    )
                    .operation_read_record(
                        operation,
                        Root::<$group>(PhantomData),
                        Entry::<$group>(PhantomData),
                        $group,
                    ))
            }
        }
        impl ApplicationSchema for Root<$group> {
            const NAME: &'static str = <Self as Fragment>::NAME;
            fn declaration(
            ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
            {
                declare(Root::<$group>(PhantomData))
            }
        }
    };
}
schema!(Form);
schema!(Reordered);
