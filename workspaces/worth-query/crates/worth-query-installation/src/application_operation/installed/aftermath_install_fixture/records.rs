//! Inverse fixtures select real columns from one owned model.
use worth_query_declaration::facade::application_schema::{fragment::*, *};
pub(crate) struct AftermathFields;
impl Fragment for AftermathFields {
    const NAME: &'static str = "aftermath_fields";
    type Bindings = Bindings;
    type Members = (Account, OtherAccount);
}
pub(crate) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
pub(crate) struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (State, OtherState);
    type Relations = ();
}
pub(crate) struct OtherAccount;
impl Entity for OtherAccount {
    const NAME: &'static str = "OtherAccount";
    type Aspects = (OtherAccountState,);
    type Relations = ();
}
pub(crate) struct State;
impl Aspect for State {
    const NAME: &'static str = "State";
    type Row = AccountState;
}
pub(crate) struct OtherState;
impl Aspect for OtherState {
    const NAME: &'static str = "OtherState";
    type Row = BalanceValue;
}
pub(crate) struct OtherAccountState;
impl Aspect for OtherAccountState {
    const NAME: &'static str = "State";
    type Row = BalanceValue;
}
pub(crate) struct AccountState {
    balance: u64,
    secret: u64,
    other_balance: u64,
}
pub(crate) struct BalanceValue {
    balance: u64,
}
impl Record<AccountState> for Bindings {
    fn record() -> impl Shape<AccountState, Self> {
        field("balance", |r: &AccountState| r.balance)
            .and("secret-field", |r: &AccountState| r.secret)
            .and("other-balance", |r: &AccountState| r.other_balance)
            .build(
                |balance, secret, other_balance, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(AccountState {
                        balance,
                        secret,
                        other_balance,
                    })
                },
            )
    }
}
impl Record<BalanceValue> for Bindings {
    fn record() -> impl Shape<BalanceValue, Self> {
        field("balance", |r: &BalanceValue| r.balance).build(
            |balance, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(BalanceValue { balance })
            },
        )
    }
}
pub(crate) type Balance = FieldSelection<AftermathFields, Account, State, u64>;
pub(crate) type SecretField = FieldSelection<AftermathFields, Account, State, u64>;
pub(crate) type OtherBalance = FieldSelection<AftermathFields, Account, State, u64>;
pub(crate) type OtherAccountBalance =
    FieldSelection<AftermathFields, OtherAccount, OtherAccountState, u64>;
pub(crate) type OtherStateBalance = FieldSelection<AftermathFields, Account, OtherState, u64>;
