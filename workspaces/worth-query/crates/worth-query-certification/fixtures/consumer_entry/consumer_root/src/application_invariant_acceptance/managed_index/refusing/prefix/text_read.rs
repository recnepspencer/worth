//! Schema-bound fixture for managed_index::refusing::prefix.
//! Adversarial reads only through genuine callback-supplied Source rows.
use super::super::schema::{Item, ItemFacts, ItemName};
use super::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};

worth_query_value_binding!(Lookalike for String {
    identity: "managed-catalog.name.v1", scalar: String,
    canonical: ValueDecodeBounds::new(2048, 1024, 8, 8),
});
struct WrongField;
impl DeclaredApplicationFieldValue for WrongField {
    type Value = String;
    type Binding = Lookalike;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}
impl RequiredApplicationFieldValue for WrongField {}
impl ApplicationFieldMarkerIdentity<Catalog, Item, ItemFacts> for WrongField {
    const IDENTIFIER: &'static str =
        <ItemName as ApplicationFieldMarkerIdentity<Catalog, Item, ItemFacts>>::IDENTIFIER;
}
fn wrong<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    query::NameSlot,
    Catalog,
    Item,
    ItemFacts,
    WrongField,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("name", ApplicationFieldRef::from_schema_types())
}
fn invalid(_: WorthQueryManagedIndexTextReadDenial) -> Stop {
    Stop::Domain(PrefixIndexDenial::InvalidTopology)
}
pub(in super::super) fn inspect<I: PrefixSourceIndex<Catalog>>(
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
    let wrong_output = ApplicationQueryResultFieldRef::<
        <<I as ApplicationManagedIndexBinding<Catalog>>::SourceBinding as ApplicationQueryBinding<Catalog>>::Query,
        query::NameSlot,
        Catalog,
        Item,
        ItemFacts,
        ItemName,
        String,
        ReadWrite,
        EqualityPredicate,
        NoApplicationUnit,
    >::new("another_output", ItemName::reference());
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
