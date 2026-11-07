//! Adversarial reads only through genuine callback-supplied Source rows.
use super::super::schema::{Item, ItemFacts, ItemName};
use super::*;
use worth_query_decl::facade::{application_query::*, application_schema::*};

use super::super::schema::{CatalogName, CatalogRecords, NameBinding};
use worth_query_decl::facade::application_schema::fragment::{self, *};
struct WrongRecords;
impl Fragment for WrongRecords {
    const NAME: &'static str = <CatalogRecords as Fragment>::NAME;
    type Bindings = WrongBindings;
    type Members = (WrongItem,);
}
struct WrongItem;
impl Entity for WrongItem {
    const NAME: &'static str = <Item as Entity>::NAME;
    type Aspects = (WrongFacts,);
    type Relations = ();
}
struct WrongFacts;
impl Aspect for WrongFacts {
    const NAME: &'static str = <ItemFacts as Aspect>::NAME;
    type Row = WrongName;
}
struct WrongName(CatalogName);
struct WrongBindings;
impl BindingSet for WrongBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<CatalogName> for WrongBindings {
    type Binding = Lookalike;
}
impl fragment::Record<WrongName> for WrongBindings {
    fn record() -> impl Shape<WrongName, Self> {
        field("ItemName", |value: &WrongName| value.0.clone())
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(WrongName(value))
            })
    }
}
struct Lookalike;
impl ApplicationScalarValueBinding for Lookalike {
    type Value = CatalogName;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = NameBinding::IDENTITY_NAME;
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        NameBinding::SCALAR_FAMILY;
    const SUPPORTS_ADMITTED_DECODE: bool = NameBinding::SUPPORTS_ADMITTED_DECODE;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        NameBinding::canonical_contract()
    }
    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        NameBinding::validate(value)
    }
    fn encode(value: &Self::Value) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        NameBinding::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for Lookalike {
    fn decode(raw: &ApplicationValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        NameBinding::decode(raw)
    }
    fn decode_admitted(
        raw: &ApplicationValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        NameBinding::decode_admitted(raw, work)
    }
}
type WrongField = FieldSelection<WrongRecords, WrongItem, WrongFacts, CatalogName>;
fn wrong<Q>() -> ApplicationQueryResultFieldRef<Q, query::NameSlot, Catalog, WrongField> {
    ApplicationQueryResultFieldRef::new("name", WrongField::at("ItemName"))
}
fn invalid(_: WorthQueryManagedIndexTextReadDenial) -> Stop {
    Stop::Domain(PrefixIndexDenial::InvalidTopology)
}
pub(super) fn inspect<I: PrefixSourceIndex>(
    calls: &Calls,
    row: &WorthQueryManagedIndexSourceRow<'_, Catalog, I>,
) -> Result<(), Stop> {
    if !calls.inspect_text.load(Ordering::SeqCst) {
        return Ok(());
    }
    row.with_text(query::name(), |first| {
        assert!(
            !first.is_empty(),
            "this authored witness uses nonempty names"
        );
        row.with_text(query::name(), |second| {
            assert_eq!(first, second);
            assert!(
                std::ptr::eq(first.as_ptr(), second.as_ptr()),
                "nested loans must share the same live decoded backing"
            );
        })
        .map_err(invalid)
    })
    .map_err(invalid)??;
    let wrong_output = ApplicationQueryResultFieldRef::<<<I as ApplicationManagedIndexBinding<Catalog>>::SourceBinding as ApplicationQueryBinding<Catalog>>::Query, query::NameSlot, Catalog, ItemName>::new("another_output", ItemName::at("ItemName"));
    let error = row
        .with_text(wrong_output, |_| {
            panic!("a warm value cannot admit another selector")
        })
        .unwrap_err();
    assert!(
        matches!(error, WorthQueryManagedIndexTextReadDenial::Projection(ref cause)
        if cause.kind() == WorthQueryApplicationProjectionDenialKind::FieldContractMismatch),
        "{error:?}"
    );
    row.with_text(query::name(), |text| assert!(!text.is_empty()))
        .map_err(invalid)?;
    calls.text_reads.fetch_add(1, Ordering::SeqCst);
    if calls.wrong_text_binding.load(Ordering::SeqCst) {
        assert_eq!(
            query::name::<<<I as ApplicationManagedIndexBinding<Catalog>>::SourceBinding as ApplicationQueryBinding<Catalog>>::Query>().slot_key(),
            wrong::<<<I as ApplicationManagedIndexBinding<Catalog>>::SourceBinding as ApplicationQueryBinding<Catalog>>::Query>().slot_key(),
            "nominal mismatch is not a different canonical key"
        );
        let error = row
            .with_text(wrong::<<<I as ApplicationManagedIndexBinding<Catalog>>::SourceBinding as ApplicationQueryBinding<Catalog>>::Query>(), |_| {
                panic!("a warm cache cannot substitute another Rust binding")
            })
            .unwrap_err();
        assert!(
            matches!(
                error,
                WorthQueryManagedIndexTextReadDenial::Controlled(
                    WorthQueryManagedIndexStop::ValueDecode(
                        WorthQueryValueDecodeDenial::BindingContractMismatch
                    )
                )
            ),
            "{error:?}"
        );
        return Err(invalid(error));
    }
    let observe = calls.after_text.lock().unwrap().take();
    if let Some(observe) = observe {
        observe();
        // Swallow the immediate read error; the invocation still must preserve
        // its original sticky cancellation at the final handoff.
        assert!(row
            .with_text(query::name(), |_| panic!(
                "stopped cache hit cannot invoke its body"
            ))
            .is_err());
    }
    Ok(())
}
