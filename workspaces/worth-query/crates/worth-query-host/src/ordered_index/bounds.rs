//! Slot-backed ordered summary bounds and metered field/UTF-8 comparison helpers.
//!
//! Eliminates nested `with_text` closure byte-checkpoint boilerplate while preserving
//! original controlled stops (`ValueDecode`, `WorkCeiling`, `Projection`) without
//! uncharged allocations.

use super::WorthQueryOrderedIndexSources;
use std::cmp::Ordering;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef,
    },
    application_schema::{fragment::RecordSelection, ApplicationSchema},
    portable_identity::WorthQueryPortableType,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionDenialKind,
    WorthQueryManagedIndexControl, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop,
    WorthQueryManagedIndexTextReadDenial,
};
use worth_query_installation::facade::ApplicationReadableScalarValueBinding;

type SourceQuery<S, I> =
    <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
type Stop<D> = WorthQueryManagedIndexProviderStop<D>;

/// Zero-allocation subtree min/max slot bounds for total-order point and prefix indexes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOrderedSlotBounds {
    lower: WorthQueryManagedIndexSourceSlot,
    upper: WorthQueryManagedIndexSourceSlot,
}

impl WorthQueryOrderedSlotBounds {
    /// Constructs a singleton enclosure for one genuine source slot.
    pub const fn point(slot: WorthQueryManagedIndexSourceSlot) -> Self {
        Self {
            lower: slot,
            upper: slot,
        }
    }

    /// Returns the lower-bound source slot of this subtree summary.
    pub const fn lower(&self) -> WorthQueryManagedIndexSourceSlot {
        self.lower
    }

    /// Returns the upper-bound source slot of this subtree summary.
    pub const fn upper(&self) -> WorthQueryManagedIndexSourceSlot {
        self.upper
    }

    /// Associatively merges two slot-bound summaries under the supplied metered row comparator.
    /// Complexity: O(compare_rows) with at most 2 comparisons (short-circuiting equal slots).
    pub fn merge_with<S, I, D>(
        &self,
        other: &Self,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
        mut compare: impl FnMut(
            WorthQueryManagedIndexSourceRow<'_, S, I>,
            WorthQueryManagedIndexSourceRow<'_, S, I>,
            &WorthQueryManagedIndexControl<'_>,
        ) -> Result<Ordering, Stop<D>>,
    ) -> Result<Self, Stop<D>>
    where
        S: ApplicationSchema,
        I: ApplicationManagedIndexBinding<S>,
    {
        control.checkpoint(1)?;
        let lower = if self.lower == other.lower {
            self.lower
        } else {
            let a = sources.get(self.lower)?;
            let b = sources.get(other.lower)?;
            if compare(a, b, control)?.then_with(|| self.lower.cmp(&other.lower))
                == Ordering::Greater
            {
                other.lower
            } else {
                self.lower
            }
        };
        let upper = if self.upper == other.upper {
            self.upper
        } else {
            let a = sources.get(self.upper)?;
            let b = sources.get(other.upper)?;
            if compare(a, b, control)?.then_with(|| self.upper.cmp(&other.upper)) == Ordering::Less
            {
                other.upper
            } else {
                self.upper
            }
        };
        Ok(Self { lower, upper })
    }
}

/// Byte normalization policy for metered UTF-8 ordering and prefix matching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrderedTextOrder {
    /// Exact UTF-8 byte ordering without case folding or allocation.
    ExactUtf8,
    /// ASCII-lowercased UTF-8 byte ordering (preserves UTF-8 character boundaries, zero allocation).
    AsciiCaseInsensitive,
}

impl WorthQueryOrderedTextOrder {
    #[inline]
    const fn normalize(self, byte: u8) -> u8 {
        match self {
            Self::ExactUtf8 => byte,
            Self::AsciiCaseInsensitive => byte.to_ascii_lowercase(),
        }
    }
}

/// Ordering of a candidate text key relative to a prefix probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrderedPrefixOrder {
    Before,
    Matches,
    After,
}

/// Translates a projection denial into a latched controlled provider stop without swallowing
/// original `ValueDecode` / `WorkCeiling` refusals.
pub fn map_projection_denial<D>(
    cause: WorthQueryApplicationProjectionDenial,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Stop<D> {
    let stop = match cause.kind() {
        WorthQueryApplicationProjectionDenialKind::ValueDecode(decode) => {
            WorthQueryManagedIndexStop::ValueDecode(decode)
        }
        _ => WorthQueryManagedIndexStop::Projection(cause),
    };
    Stop::Controlled(control.refuse(stop))
}

/// Translates a controlled text-read denial into a latched controlled provider stop.
pub fn map_text_read_denial<D>(
    cause: WorthQueryManagedIndexTextReadDenial,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Stop<D> {
    match cause {
        WorthQueryManagedIndexTextReadDenial::Controlled(stop) => {
            Stop::Controlled(control.refuse(stop))
        }
        WorthQueryManagedIndexTextReadDenial::Projection(denial) => {
            map_projection_denial(denial, control)
        }
    }
}

/// Reads one projected field from a genuine source row with metered checkpointing and
/// lossless controlled-stop translation.
/// Complexity: O(field decode cost).
pub fn read_ordered_field<S, I, FieldSlot, Entity, Aspect, Field, Value, D>(
    row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Value, Stop<D>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
    Field::Binding: ApplicationReadableScalarValueBinding,
    FieldSlot: WorthQueryPortableType,
{
    control.checkpoint(1)?;
    row.field(selector)
        .map_err(|cause| map_projection_denial(cause, control))
}

/// Metered byte-by-byte UTF-8 comparison charging 1 work unit per compared byte.
/// Complexity: O(min(|left|, |right|)) time, O(1) space.
pub fn compare_text_metered<D>(
    left: &str,
    right: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Ordering, Stop<D>> {
    for (a, b) in left.bytes().zip(right.bytes()) {
        control.checkpoint(1)?;
        let cmp = order.normalize(a).cmp(&order.normalize(b));
        if cmp != Ordering::Equal {
            return Ok(cmp);
        }
    }
    control.checkpoint(1)?;
    Ok(left.len().cmp(&right.len()))
}

/// Metered prefix ordering (`Before` / `Matches` / `After`) charging 1 work unit per probe byte.
/// Complexity: O(|prefix|) time, O(1) space.
pub fn prefix_order_metered<D>(
    key: &str,
    prefix: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<WorthQueryOrderedPrefixOrder, Stop<D>> {
    control.checkpoint(1)?;
    let key_bytes = key.as_bytes();
    for (index, prefix_byte) in prefix.bytes().enumerate() {
        control.checkpoint(1)?;
        let Some(&key_byte) = key_bytes.get(index) else {
            return Ok(WorthQueryOrderedPrefixOrder::Before);
        };
        match order.normalize(key_byte).cmp(&order.normalize(prefix_byte)) {
            Ordering::Less => return Ok(WorthQueryOrderedPrefixOrder::Before),
            Ordering::Greater => return Ok(WorthQueryOrderedPrefixOrder::After),
            Ordering::Equal => {}
        }
    }
    Ok(WorthQueryOrderedPrefixOrder::Matches)
}

/// Sound prefix pruning check over `[lower, upper]` text bounds.
/// Complexity: O(|prefix|) time, O(1) space.
pub fn prefix_bounds_may_match_metered<D>(
    lower: &str,
    upper: &str,
    prefix: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop<D>> {
    Ok(
        prefix_order_metered(lower, prefix, order, control)? != WorthQueryOrderedPrefixOrder::After
            && prefix_order_metered(upper, prefix, order, control)?
                != WorthQueryOrderedPrefixOrder::Before,
    )
}

/// Sound inverse-watch pruning check asking whether any UTF-8 prefix of `row` lies in `[lower, upper]`.
/// Complexity: O(|lower| + |upper|) time, O(1) space.
pub fn prefix_watch_bounds_may_match_metered<D>(
    lower: &str,
    upper: &str,
    row: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop<D>> {
    control.checkpoint(1)?;
    let row_bytes = row.as_bytes();
    for (index, lower_byte) in lower.bytes().enumerate() {
        control.checkpoint(1)?;
        let Some(&row_byte) = row_bytes.get(index) else {
            return Ok(false);
        };
        match order.normalize(row_byte).cmp(&order.normalize(lower_byte)) {
            Ordering::Less => return Ok(false),
            Ordering::Greater => {
                let mut end = index + 1;
                while !row.is_char_boundary(end) {
                    control.checkpoint(1)?;
                    end += 1;
                }
                return Ok(
                    compare_text_metered(&row[..end], upper, order, control)? != Ordering::Greater
                );
            }
            Ordering::Equal => {}
        }
    }
    Ok(true)
}

/// Compares one projected text field across two genuine source rows using borrow-confined
/// `with_text` loans and metered byte checkpoints.
pub fn compare_text_fields<S, I, FieldSlot, Entity, Aspect, Field, D>(
    left: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    right: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Ordering, Stop<D>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect>,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType,
{
    left.with_text(selector, |a| {
        right
            .with_text(selector, |b| compare_text_metered(a, b, order, control))
            .map_err(|cause| map_text_read_denial(cause, control))?
    })
    .map_err(|cause| map_text_read_denial(cause, control))?
}

/// Evaluates whether a row's projected text field starts with `prefix` under `order`.
pub fn prefix_matches_text_field<S, I, FieldSlot, Entity, Aspect, Field, D>(
    row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    prefix: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop<D>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect>,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType,
{
    let ord = row
        .with_text(selector, |text| {
            prefix_order_metered(text, prefix, order, control)
        })
        .map_err(|cause| map_text_read_denial(cause, control))??;
    Ok(ord == WorthQueryOrderedPrefixOrder::Matches)
}

/// Soundly tests whether any row enclosed by `bounds` may match `prefix` on `selector`.
pub fn prefix_bounds_may_match_field<S, I, FieldSlot, Entity, Aspect, Field, D>(
    bounds: &WorthQueryOrderedSlotBounds,
    sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
    selector: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    prefix: &str,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop<D>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect>,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType,
{
    let lower = sources.get(bounds.lower())?;
    let upper = sources.get(bounds.upper())?;
    lower
        .with_text(selector, |lo| {
            upper
                .with_text(selector, |hi| {
                    prefix_bounds_may_match_metered(lo, hi, prefix, order, control)
                })
                .map_err(|cause| map_text_read_denial(cause, control))?
        })
        .map_err(|cause| map_text_read_denial(cause, control))?
}

/// Soundly tests whether any watch probe in `[extrema[0], extrema[1]]` may match `row`'s `selector`.
pub fn prefix_watch_bounds_may_match_field<S, I, FieldSlot, Entity, Aspect, Field, D>(
    extrema: [&String; 2],
    row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
    selector: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    order: WorthQueryOrderedTextOrder,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop<D>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect>,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType,
{
    row.with_text(selector, |text| {
        prefix_watch_bounds_may_match_metered(
            extrema[0].as_str(),
            extrema[1].as_str(),
            text,
            order,
            control,
        )
    })
    .map_err(|cause| map_text_read_denial(cause, control))?
}
