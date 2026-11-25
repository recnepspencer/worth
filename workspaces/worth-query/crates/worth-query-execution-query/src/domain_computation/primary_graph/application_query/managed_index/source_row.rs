//! A genuine Source row with controlled, borrow-confined content and text.
mod content;
use super::{
    source_text::SourceText, WorthQueryManagedIndexControl as Control,
    WorthQueryManagedIndexSourceSlot as Slot, WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::primary_graph::{
    application_query::projection::{
        WorthQueryApplicationProjectedField, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    value_decode::ValueDecodeSession,
};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryMarkerIdentity,
        ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    },
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
    portable_identity::WorthQueryPortableType,
};
use worth_query_installation::facade::ApplicationReadableScalarValueBinding;

type Query<S, I> =
    <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;

/// A controlled read refused either its actual projection or original request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexTextReadDenial {
    Projection(WorthQueryApplicationProjectionDenial),
    Controlled(Stop),
}
impl From<Stop> for WorthQueryManagedIndexTextReadDenial {
    fn from(cause: Stop) -> Self {
        Self::Controlled(cause)
    }
}
impl From<WorthQueryApplicationProjectionDenial> for WorthQueryManagedIndexTextReadDenial {
    fn from(cause: WorthQueryApplicationProjectionDenial) -> Self {
        Self::Projection(cause)
    }
}

/// The installed identity of one lent text field.
struct TextKey<'key> {
    entity: &'key str,
    aspect: &'key str,
    field: &'key str,
    output: &'key str,
    slot: &'key str,
}

/// A row selected and admitted from one actual immutable Source image.
/// Construction and the memo are private; ordinary projection methods remain
/// available. Text is lent only for the duration of a controlled callback.
pub struct WorthQueryManagedIndexSourceRow<
    'a,
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    row: WorthQueryApplicationProjectionRow<'a, S, Query<S, I>>,
    slot: Slot,
    text: &'a SourceText,
    decode: &'a ValueDecodeSession<'a>,
    control: &'a Control<'a>,
    execution: ExecutionRequest<'a, 'a>,
}
impl<'a, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSourceRow<'a, S, I>
{
    pub(crate) fn new(
        row: WorthQueryApplicationProjectionRow<'a, S, Query<S, I>>,
        slot: Slot,
        text: &'a SourceText,
        decode: &'a ValueDecodeSession<'a>,
        control: &'a Control<'a>,
        execution: ExecutionRequest<'a, 'a>,
    ) -> Self {
        Self {
            row,
            slot,
            text,
            decode,
            control,
            execution,
        }
    }

    /// Lend admitted decoded text without copying or allowing its storage
    /// to escape the original invocation's parse reservation.
    pub fn with_text<FieldSlot, Entity, Aspect, Field, Output>(
        &self,
        selector: ApplicationQueryResultFieldRef<Query<S, I>, FieldSlot, S, Field>,
        body: impl FnOnce(&str) -> Output,
    ) -> Result<Output, WorthQueryManagedIndexTextReadDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Field::Value: Into<String>,

        FieldSlot: WorthQueryPortableType,
    {
        let slot = FieldSlot::PORTABLE_TYPE_IDENTITY;
        let key = TextKey {
            entity: selector.entity(),
            aspect: selector.aspect(),
            field: selector.field(),
            output: selector.output_name(),
            slot: slot.as_str(),
        };
        let key_work = self.admit_text_key::<Field::Binding>(&key)?;
        let projected = self.row.required_field(selector)?;
        self.lend_text::<Field::Binding, _>(&key, key_work, projected, body)
    }

    /// Lend an Optional field's admitted decoded text under the same controls
    /// as [`Self::with_text`]; a row without a value lends nothing (`None`).
    pub fn with_optional_text<FieldSlot, Entity, Aspect, Field, Output>(
        &self,
        selector: ApplicationQueryOptionalResultFieldRef<Query<S, I>, FieldSlot, S, Field>,
        body: impl FnOnce(&str) -> Output,
    ) -> Result<Option<Output>, WorthQueryManagedIndexTextReadDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Field::Value: Into<String>,
        FieldSlot: WorthQueryPortableType,
    {
        let slot = FieldSlot::PORTABLE_TYPE_IDENTITY;
        let key = TextKey {
            entity: selector.entity(),
            aspect: selector.aspect(),
            field: selector.field(),
            output: selector.output_name(),
            slot: slot.as_str(),
        };
        let key_work = self.admit_text_key::<Field::Binding>(&key)?;
        let Some(projected) = self.row.optional_projected_field(selector)? else {
            return Ok(None);
        };
        self.lend_text::<Field::Binding, _>(&key, key_work, projected, body)
            .map(Some)
    }

    /// Charges the canonical comparison work of one text key: borrowed
    /// portable identities as well as all four field names (retained heap
    /// bytes alone omit them).
    fn admit_text_key<Binding: ApplicationScalarValueBinding>(
        &self,
        key: &TextKey<'_>,
    ) -> Result<usize, WorthQueryManagedIndexTextReadDenial> {
        self.check()?;
        self.control.checkpoint(1)?;
        let key_work = [
            key.entity,
            key.aspect,
            key.field,
            key.output,
            Query::<S, I>::QUERY_TYPE_IDENTITY.as_str(),
            key.slot,
            Binding::IDENTITY.as_str(),
        ]
        .into_iter()
        .try_fold(size_of::<Slot>() + size_of::<usize>(), |sum, part| {
            sum.checked_add(part.len())
        })
        .ok_or_else(|| self.control.refuse(Stop::CapacityOverflow))?;
        let admission_work = u64::try_from(key_work)
            .ok()
            .and_then(|work| work.checked_mul(2))
            .ok_or_else(|| self.control.refuse(Stop::CapacityOverflow))?;
        self.control.checkpoint(admission_work)?;
        Ok(key_work)
    }

    fn lend_text<Binding, Output>(
        &self,
        key: &TextKey<'_>,
        key_work: usize,
        projected: &WorthQueryApplicationProjectedField,
        body: impl FnOnce(&str) -> Output,
    ) -> Result<Output, WorthQueryManagedIndexTextReadDenial>
    where
        Binding: ApplicationReadableScalarValueBinding,
        Binding::Value: Into<String>,
    {
        let slot_key = projected.slot_key_identity();
        let binding =
            self.text
                .binding(slot_key, key_work, &self.execution, self.control, || {
                    self.decode
                        .shared_field(key.entity, key.aspect, key.field)
                        .ok_or_else(|| self.decode_stop())
                })?;
        if !self.decode.verify_binding::<Binding>(&binding) {
            return Err(self.decode_stop().into());
        }
        let text = self.text.text(
            self.slot,
            slot_key,
            key_work,
            &self.execution,
            self.control,
            || {
                self.decode
                    .decode_installed::<Binding>(&binding, projected.value())
                    .map(Into::into)
                    .ok_or_else(|| self.decode_stop())
            },
        )?;
        // No RefCell guard survives into application code, including nested reads.
        let output = body(text.as_str());
        self.check()?;
        Ok(output)
    }
    fn decode_stop(&self) -> Stop {
        self.control.refuse(Stop::ValueDecode(
            self.decode
                .denial()
                .expect("failed installed decode latches refusal"),
        ))
    }
    fn check(&self) -> Result<(), Stop> {
        if self.decode.denial().is_some() {
            return Err(self.decode_stop());
        }
        self.control.poll()
    }
}
impl<'a, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> std::ops::Deref
    for WorthQueryManagedIndexSourceRow<'a, S, I>
{
    type Target = WorthQueryApplicationProjectionRow<'a, S, Query<S, I>>;
    fn deref(&self) -> &Self::Target {
        &self.row
    }
}

impl std::fmt::Display for WorthQueryManagedIndexTextReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "managed Source text read denied: {self:?}")
    }
}
impl std::error::Error for WorthQueryManagedIndexTextReadDenial {}
