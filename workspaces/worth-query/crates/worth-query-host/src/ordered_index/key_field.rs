//! Declared Source fields a stock ordered index keys on.
//!
//! A Required field selector always yields a key. An Optional field selector
//! makes the index sparse: a row whose key (or any interval or box endpoint)
//! is absent contributes the empty summary, never matches a probe and never
//! routes a watch. It keeps its structural node, because the checked binary
//! source tree holds every Source row exactly once.
use super::bounds::{map_projection_denial, map_text_read_denial, read_ordered_field};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryMarkerIdentity,
        ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationReadableScalarValueBinding, ApplicationSchema,
    },
    portable_identity::WorthQueryPortableType,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexSourceRow,
};

type Stop<D> = WorthQueryManagedIndexProviderStop<D>;

mod sealed {
    pub trait Sealed {}
}

/// A declared Required or Optional field whose value keys a stock ordered index.
pub trait WorthQueryOrderedKeyField<S, I>: sealed::Sealed + Copy + Send + Sync + 'static
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    /// The decoded key value.
    type Value;

    /// Reads this row's key; `None` when an Optional field carries no value.
    fn read_key<D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<Self::Value>, Stop<D>>;
}

/// A declared Required or Optional text field lent to a stock prefix index.
pub trait WorthQueryOrderedTextKeyField<S, I>:
    sealed::Sealed + Copy + Send + Sync + 'static
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    /// Lends this row's key text to `body`; `None` when an Optional field
    /// carries no value. The text cannot escape the callback.
    fn with_key_text<Output, D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
        body: impl FnOnce(&str) -> Result<Output, Stop<D>>,
    ) -> Result<Option<Output>, Stop<D>>;

    /// Whether this row carries key text; a Required field always does and
    /// reads nothing.
    fn key_present<D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop<D>>;
}

impl<Q, Slot, S, Field: RecordSelection> sealed::Sealed
    for ApplicationQueryResultFieldRef<Q, Slot, S, Field>
{
}
impl<Q, Slot, S, Field: RecordSelection> sealed::Sealed
    for ApplicationQueryOptionalResultFieldRef<Q, Slot, S, Field>
{
}

impl<S, I, Q, Slot, Field> WorthQueryOrderedKeyField<S, I>
    for ApplicationQueryResultFieldRef<Q, Slot, S, Field>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    I::SourceBinding: ApplicationQueryBinding<S, Query = Q>,
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Field: RecordSelection,
    Slot: WorthQueryPortableType + 'static,
{
    type Value = Field::Value;

    fn read_key<D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<Field::Value>, Stop<D>> {
        read_ordered_field(row, *self, control).map(Some)
    }
}

impl<S, I, Q, Slot, Field> WorthQueryOrderedKeyField<S, I>
    for ApplicationQueryOptionalResultFieldRef<Q, Slot, S, Field>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    I::SourceBinding: ApplicationQueryBinding<S, Query = Q>,
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Field: RecordSelection,
    Slot: WorthQueryPortableType + 'static,
{
    type Value = Field::Value;

    fn read_key<D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<Field::Value>, Stop<D>> {
        control.checkpoint(1)?;
        row.optional_field(*self)
            .map_err(|cause| map_projection_denial(cause, control))
    }
}

impl<S, I, Q, Slot, Field> WorthQueryOrderedTextKeyField<S, I>
    for ApplicationQueryResultFieldRef<Q, Slot, S, Field>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    I::SourceBinding: ApplicationQueryBinding<S, Query = Q>,
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Field: RecordSelection,
    Field::Value: Into<String>,
    Slot: WorthQueryPortableType + 'static,
{
    fn with_key_text<Output, D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
        body: impl FnOnce(&str) -> Result<Output, Stop<D>>,
    ) -> Result<Option<Output>, Stop<D>> {
        row.with_text(*self, body)
            .map_err(|cause| map_text_read_denial(cause, control))?
            .map(Some)
    }

    fn key_present<D>(
        &self,
        _row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        _control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop<D>> {
        Ok(true)
    }
}

impl<S, I, Q, Slot, Field> WorthQueryOrderedTextKeyField<S, I>
    for ApplicationQueryOptionalResultFieldRef<Q, Slot, S, Field>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    I::SourceBinding: ApplicationQueryBinding<S, Query = Q>,
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Field: RecordSelection,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    Slot: WorthQueryPortableType + 'static,
{
    fn with_key_text<Output, D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
        body: impl FnOnce(&str) -> Result<Output, Stop<D>>,
    ) -> Result<Option<Output>, Stop<D>> {
        row.with_optional_text(*self, body)
            .map_err(|cause| map_text_read_denial(cause, control))?
            .transpose()
    }

    fn key_present<D>(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop<D>> {
        Ok(self.with_key_text(row, control, |_| Ok(()))?.is_some())
    }
}
