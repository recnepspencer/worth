//! Stock checked 2D AABB overlap managed-index adapter built on
//! [`WorthQueryOrderedIndexAdapter`].
use super::box_nd::impl_box_domain;
use super::coord::{WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion};
use super::probes::WorthQueryBox2DProbe;
use super::WorthQueryOrderedIndexAdapter;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef,
    },
    application_schema::{fragment::RecordSelection, ApplicationSchema},
};

type SourceQuery<S, I> =
    <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;
type FieldRef<S, I, Slot, Field> =
    ApplicationQueryResultFieldRef<SourceQuery<S, I>, Slot, S, Field>;

/// Fixed-size `Copy` associative 2D AABB envelope summary.
pub type WorthQueryBox2DEnvelope<C> = WorthQueryAxisEnvelope<C, 2>;

/// Stock checked 2D bounding-box domain over declared `(min_x, min_y, max_x, max_y)` fields.
pub struct WorthQueryBox2DDomain<
    S,
    I,
    MinXS,
    MinXF: RecordSelection,
    MinYS,
    MinYF: RecordSelection,
    MaxXS,
    MaxXF: RecordSelection,
    MaxYS,
    MaxYF: RecordSelection,
    C,
> where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    min_x: FieldRef<S, I, MinXS, MinXF>,
    min_y: FieldRef<S, I, MinYS, MinYF>,
    max_x: FieldRef<S, I, MaxXS, MaxXF>,
    max_y: FieldRef<S, I, MaxYS, MaxYF>,
    inclusion: WorthQueryRangeInclusion,
    _marker: PhantomData<fn() -> (S, I, C)>,
}

/// Stock checked 2D AABB managed-index provider with optional inverse watch routing.
pub type WorthQueryBox2DIndex<S, I, MinXS, MinXF, MinYS, MinYF, MaxXS, MaxXF, MaxYS, MaxYF, C> =
    WorthQueryOrderedIndexAdapter<
        WorthQueryBox2DDomain<S, I, MinXS, MinXF, MinYS, MinYF, MaxXS, MaxXF, MaxYS, MaxYF, C>,
    >;

impl<
        S,
        I,
        MinXS,
        MinXF: RecordSelection,
        MinYS,
        MinYF: RecordSelection,
        MaxXS,
        MaxXF: RecordSelection,
        MaxYS,
        MaxYF: RecordSelection,
        C,
    > WorthQueryBox2DIndex<S, I, MinXS, MinXF, MinYS, MinYF, MaxXS, MaxXF, MaxYS, MaxYF, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    /// Half-open `[min_x, max_x) x [min_y, max_y)` box index.
    pub const fn half_open(
        min_x: FieldRef<S, I, MinXS, MinXF>,
        min_y: FieldRef<S, I, MinYS, MinYF>,
        max_x: FieldRef<S, I, MaxXS, MaxXF>,
        max_y: FieldRef<S, I, MaxYS, MaxYF>,
    ) -> Self {
        Self::new_with_inclusion(
            (min_x, min_y),
            (max_x, max_y),
            WorthQueryRangeInclusion::HalfOpen,
        )
    }

    /// Closed `[min_x, max_x] x [min_y, max_y]` box index.
    pub const fn closed(
        min_x: FieldRef<S, I, MinXS, MinXF>,
        min_y: FieldRef<S, I, MinYS, MinYF>,
        max_x: FieldRef<S, I, MaxXS, MaxXF>,
        max_y: FieldRef<S, I, MaxYS, MaxYF>,
    ) -> Self {
        Self::new_with_inclusion(
            (min_x, min_y),
            (max_x, max_y),
            WorthQueryRangeInclusion::Closed,
        )
    }

    /// 2D box index with explicit [`WorthQueryRangeInclusion`].
    pub const fn new_with_inclusion(
        min: (FieldRef<S, I, MinXS, MinXF>, FieldRef<S, I, MinYS, MinYF>),
        max: (FieldRef<S, I, MaxXS, MaxXF>, FieldRef<S, I, MaxYS, MaxYF>),
        inclusion: WorthQueryRangeInclusion,
    ) -> Self {
        WorthQueryOrderedIndexAdapter::new(WorthQueryBox2DDomain {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
            inclusion,
            _marker: PhantomData,
        })
    }
}

impl_box_domain!(
    WorthQueryBox2DDomain,
    WorthQueryBox2DProbe,
    box_2d_bounds,
    2,
    [(MinXS, MinXF, min_x), (MinYS, MinYF, min_y)],
    [(MaxXS, MaxXF, max_x), (MaxYS, MaxYF, max_y)]
);
