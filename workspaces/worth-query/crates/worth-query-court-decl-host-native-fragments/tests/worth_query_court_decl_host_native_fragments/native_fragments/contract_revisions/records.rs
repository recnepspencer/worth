//! Declaration variants isolate field order, names, presence and codec identity.
use super::*;
pub struct Bindings<Codec>(PhantomData<Codec>);
impl<C: 'static> BindingSet for Bindings<C> {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl<C: ApplicationReadableScalarValueBinding<Value = u64>> Scalar<u64> for Bindings<C> {
    type Binding = C;
}
impl<C: 'static> Scalar<bool> for Bindings<C> {
    type Binding = BoolApplicationValueBinding;
}
pub type Standard = Bindings<U64ApplicationValueBinding>;
pub type Alternate = Bindings<AlternateU64>;
pub struct Row {
    pub left: u64,
    pub right: bool,
}
pub struct ReorderedRow {
    pub left: u64,
    pub right: bool,
}
pub struct RenamedFieldRow {
    pub left: u64,
    pub right: bool,
}
pub struct OptionalFieldRow {
    pub left: Option<u64>,
    pub right: bool,
}
pub struct OtherRow {
    pub flag: bool,
}
macro_rules! row {
    ($row:ident, $first:literal => $left:ident, $second:literal => $right:ident) => {
        impl<C: ApplicationReadableScalarValueBinding<Value = u64>> Record<$row> for Bindings<C> {
            fn record() -> impl Shape<$row, Self> {
                field($first, |row: &$row| row.$left)
                    .and($second, |row: &$row| row.$right)
                    .build(|$left, $right, _: &mut ValueDecodeAdmission<'_>| {
                        Ok::<_, std::convert::Infallible>($row { $left, $right })
                    })
            }
        }
    };
}
row!(Row, "left" => left, "right" => right);
row!(ReorderedRow, "right" => right, "left" => left);
row!(RenamedFieldRow, "renamed_left" => left, "right" => right);
row!(OptionalFieldRow, "left" => left, "right" => right);
impl<C: 'static> Record<OtherRow> for Bindings<C> {
    fn record() -> impl Shape<OtherRow, Self> {
        field("flag", |row: &OtherRow| row.flag).build(|flag, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(OtherRow { flag })
        })
    }
}
#[allow(
    dead_code,
    reason = "This lowering-only fixture retains both lawful enum payloads without performing reads."
)]
pub enum ChoiceRow<R> {
    Main(R),
    Other(OtherRow),
}
macro_rules! choice_row {
    ($($row:ty),+) => { $(impl From<$row> for ChoiceRow<$row> { fn from(row: $row) -> Self { Self::Main(row) } })+ };
}
choice_row!(Row, ReorderedRow, RenamedFieldRow, OptionalFieldRow);
impl<R> From<OtherRow> for ChoiceRow<R> {
    fn from(row: OtherRow) -> Self {
        Self::Other(row)
    }
}

// The scalar family and executable codec are unchanged; only portable identity differs.
pub struct AlternateU64;
impl ApplicationScalarValueBinding for AlternateU64 {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "revision.alternate.u64";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::UInt64;
    const SUPPORTS_ADMITTED_DECODE: bool = U64ApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        U64ApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &u64) -> Result<(), ApplicationValueValidationDenial> {
        U64ApplicationValueBinding::validate(value)
    }
    fn encode(value: &u64) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        U64ApplicationValueBinding::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for AlternateU64 {
    fn decode(value: &AspectValue) -> Result<u64, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode(value)
    }
    fn decode_admitted(
        value: &AspectValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<u64, ApplicationValueDecodeDenial> {
        U64ApplicationValueBinding::decode_admitted(value, admission)
    }
}
impl ApplicationIdentityScalarValueBinding for AlternateU64 {}
