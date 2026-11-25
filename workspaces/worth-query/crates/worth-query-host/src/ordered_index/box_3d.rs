//! Stock checked 3D AABB overlap managed-index adapter built on
//! [`WorthQueryOrderedIndexAdapter`]. Any corner coordinate may be an
//! Optional field: a row with an absent coordinate is sparse.
use super::box_nd::impl_box_domain;
use super::coord::{WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion};
use super::probes::WorthQueryBox3DProbe;
use super::WorthQueryOrderedIndexAdapter;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

/// Fixed-size `Copy` associative 3D AABB envelope summary.
pub type WorthQueryBox3DEnvelope<C> = WorthQueryAxisEnvelope<C, 3>;

/// Stock checked 3D bounding-box domain over declared Required or Optional
/// `(min_x, min_y, min_z, max_x, max_y, max_z)` fields.
pub struct WorthQueryBox3DDomain<S, I, MinX, MinY, MinZ, MaxX, MaxY, MaxZ, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    min_x: MinX,
    min_y: MinY,
    min_z: MinZ,
    max_x: MaxX,
    max_y: MaxY,
    max_z: MaxZ,
    inclusion: WorthQueryRangeInclusion,
    _marker: PhantomData<fn() -> (S, I, C)>,
}

/// Stock checked 3D AABB managed-index provider with optional inverse watch routing.
pub type WorthQueryBox3DIndex<S, I, MinX, MinY, MinZ, MaxX, MaxY, MaxZ, C> =
    WorthQueryOrderedIndexAdapter<
        WorthQueryBox3DDomain<S, I, MinX, MinY, MinZ, MaxX, MaxY, MaxZ, C>,
    >;

impl<S, I, MinX, MinY, MinZ, MaxX, MaxY, MaxZ, C>
    WorthQueryBox3DIndex<S, I, MinX, MinY, MinZ, MaxX, MaxY, MaxZ, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    /// Half-open `[min_x, max_x) x [min_y, max_y) x [min_z, max_z)` box index.
    pub fn half_open(min: (MinX, MinY, MinZ), max: (MaxX, MaxY, MaxZ)) -> Self {
        Self::new_with_inclusion(min, max, WorthQueryRangeInclusion::HalfOpen)
    }

    /// Closed `[min_x, max_x] x [min_y, max_y] x [min_z, max_z]` box index.
    pub fn closed(min: (MinX, MinY, MinZ), max: (MaxX, MaxY, MaxZ)) -> Self {
        Self::new_with_inclusion(min, max, WorthQueryRangeInclusion::Closed)
    }

    /// 3D box index with explicit [`WorthQueryRangeInclusion`].
    pub fn new_with_inclusion(
        min: (MinX, MinY, MinZ),
        max: (MaxX, MaxY, MaxZ),
        inclusion: WorthQueryRangeInclusion,
    ) -> Self {
        WorthQueryOrderedIndexAdapter::new(WorthQueryBox3DDomain {
            min_x: min.0,
            min_y: min.1,
            min_z: min.2,
            max_x: max.0,
            max_y: max.1,
            max_z: max.2,
            inclusion,
            _marker: PhantomData,
        })
    }
}

impl_box_domain!(
    WorthQueryBox3DDomain,
    WorthQueryBox3DProbe,
    box_3d_bounds,
    3,
    [(MinX, min_x), (MinY, min_y), (MinZ, min_z)],
    [(MaxX, max_x), (MaxY, max_y), (MaxZ, max_z)]
);
