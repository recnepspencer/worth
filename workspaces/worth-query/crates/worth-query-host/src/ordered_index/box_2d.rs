//! Stock checked 2D AABB overlap managed-index adapter built on
//! [`WorthQueryOrderedIndexAdapter`]. Any corner coordinate may be an
//! Optional field: a row with an absent coordinate is sparse.
use super::box_nd::impl_box_domain;
use super::coord::{WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion};
use super::probes::WorthQueryBox2DProbe;
use super::WorthQueryOrderedIndexAdapter;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

/// Fixed-size `Copy` associative 2D AABB envelope summary.
pub type WorthQueryBox2DEnvelope<C> = WorthQueryAxisEnvelope<C, 2>;

/// Stock checked 2D bounding-box domain over declared Required or Optional
/// `(min_x, min_y, max_x, max_y)` fields.
pub struct WorthQueryBox2DDomain<S, I, MinX, MinY, MaxX, MaxY, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    min_x: MinX,
    min_y: MinY,
    max_x: MaxX,
    max_y: MaxY,
    inclusion: WorthQueryRangeInclusion,
    _marker: PhantomData<fn() -> (S, I, C)>,
}

/// Stock checked 2D AABB managed-index provider with optional inverse watch routing.
pub type WorthQueryBox2DIndex<S, I, MinX, MinY, MaxX, MaxY, C> =
    WorthQueryOrderedIndexAdapter<WorthQueryBox2DDomain<S, I, MinX, MinY, MaxX, MaxY, C>>;

impl<S, I, MinX, MinY, MaxX, MaxY, C> WorthQueryBox2DIndex<S, I, MinX, MinY, MaxX, MaxY, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    /// Half-open `[min_x, max_x) x [min_y, max_y)` box index.
    pub fn half_open(min_x: MinX, min_y: MinY, max_x: MaxX, max_y: MaxY) -> Self {
        Self::new_with_inclusion(
            (min_x, min_y),
            (max_x, max_y),
            WorthQueryRangeInclusion::HalfOpen,
        )
    }

    /// Closed `[min_x, max_x] x [min_y, max_y]` box index.
    pub fn closed(min_x: MinX, min_y: MinY, max_x: MaxX, max_y: MaxY) -> Self {
        Self::new_with_inclusion(
            (min_x, min_y),
            (max_x, max_y),
            WorthQueryRangeInclusion::Closed,
        )
    }

    /// 2D box index with explicit [`WorthQueryRangeInclusion`].
    pub fn new_with_inclusion(
        min: (MinX, MinY),
        max: (MaxX, MaxY),
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
    [(MinX, min_x), (MinY, min_y)],
    [(MaxX, max_x), (MaxY, max_y)]
);
