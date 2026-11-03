//! A genuine Source row with controlled, borrow-confined content and text.
mod content;
use super::{
    source_text::SourceText, WorthQueryManagedIndexControl as Control,
    WorthQueryManagedIndexSourceSlot as Slot, WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::primary_graph::{
    application_query::projection::{
        WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
    },
    value_decode::ValueDecodeSession,
};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryMarkerIdentity,
        ApplicationQueryResultFieldRef,
    },
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
    portable_identity::WorthQueryPortableType,
};
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, RequiredApplicationFieldValue,
    WritePosture,
};

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

/// A row selected and admitted from one actual immutable Source image.
/// Construction and the memo are private; ordinary projection methods remain
/// available. Text is lent only for the duration of a controlled callback.
#[doc = include_str!("source_row.md")]
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
    pub(super) fn new(
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

    /// Lend an admitted decoded String without copying or allowing its storage
    /// to escape the original invocation's parse reservation.
    pub fn with_text<FieldSlot, Entity, Aspect, Field, Write, Equality, Unit, Output>(
        &self,
        selector: ApplicationQueryResultFieldRef<
            Query<S, I>,
            FieldSlot,
            S,
            Entity,
            Aspect,
            Field,
            String,
            Write,
            Equality,
            Unit,
        >,
        body: impl FnOnce(&str) -> Output,
    ) -> Result<Output, WorthQueryManagedIndexTextReadDenial>
    where
        Field: RequiredApplicationFieldValue<Value = String>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        FieldSlot: WorthQueryPortableType,
    {
        self.check()?;
        self.control.checkpoint(1)?;
        // Canonical comparison work includes borrowed portable identities as
        // well as all four field names; retained heap bytes alone omit them.
        let key_work = [
            selector.entity(),
            selector.aspect(),
            selector.field(),
            selector.output_name(),
            Query::<S, I>::QUERY_TYPE_IDENTITY.as_str(),
            FieldSlot::PORTABLE_TYPE_IDENTITY.as_str(),
            Field::Binding::IDENTITY.as_str(),
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
        let projected = self.row.required_field(selector)?;
        let key = projected.slot_key_identity();
        let binding = self
            .text
            .binding(key, key_work, &self.execution, self.control, || {
                self.decode
                    .shared_field(selector.entity(), selector.aspect(), selector.field())
                    .ok_or_else(|| self.decode_stop())
            })?;
        if !self.decode.verify_binding::<Field::Binding>(&binding) {
            return Err(self.decode_stop().into());
        }
        let text = self.text.text(
            self.slot,
            key,
            key_work,
            &self.execution,
            self.control,
            || {
                self.decode
                    .decode_installed::<Field::Binding>(&binding, projected.value())
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
