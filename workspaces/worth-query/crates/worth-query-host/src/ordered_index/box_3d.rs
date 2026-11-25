//! Stock checked 3D AABB overlap managed-index adapter built on
//! [`WorthQueryOrderedIndexAdapter`].
use super::box_nd::impl_box_domain;
use super::coord::{WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion};
use super::probes::WorthQueryBox3DProbe;
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

/// Fixed-size `Copy` associative 3D AABB envelope summary.
pub type WorthQueryBox3DEnvelope<C> = WorthQueryAxisEnvelope<C, 3>;

/// Stock checked 3D bounding-box domain over declared `(min_x, min_y, min_z, max_x, max_y, max_z)` fields.
#[allow(clippy::type_complexity)]
pub struct WorthQueryBox3DDomain<
    S,
    I,
    MinXS,
    MinXF: RecordSelection,
    MinYS,
    MinYF: RecordSelection,
    MinZS,
    MinZF: RecordSelection,
    MaxXS,
    MaxXF: RecordSelection,
    MaxYS,
    MaxYF: RecordSelection,
    MaxZS,
    MaxZF: RecordSelection,
    C,
> where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    min_x: FieldRef<S, I, MinXS, MinXF>,
    min_y: FieldRef<S, I, MinYS, MinYF>,
    min_z: FieldRef<S, I, MinZS, MinZF>,
    max_x: FieldRef<S, I, MaxXS, MaxXF>,
    max_y: FieldRef<S, I, MaxYS, MaxYF>,
    max_z: FieldRef<S, I, MaxZS, MaxZF>,
    inclusion: WorthQueryRangeInclusion,
    _marker: PhantomData<fn() -> (S, I, C)>,
}

/// Stock checked 3D AABB managed-index provider with optional inverse watch routing.
#[allow(clippy::type_complexity)]
pub type WorthQueryBox3DIndex<
    S,
    I,
    MinXS,
    MinXF,
    MinYS,
    MinYF,
    MinZS,
    MinZF,
    MaxXS,
    MaxXF,
    MaxYS,
    MaxYF,
    MaxZS,
    MaxZF,
    C,
> = WorthQueryOrderedIndexAdapter<
    WorthQueryBox3DDomain<
        S,
        I,
        MinXS,
        MinXF,
        MinYS,
        MinYF,
        MinZS,
        MinZF,
        MaxXS,
        MaxXF,
        MaxYS,
        MaxYF,
        MaxZS,
        MaxZF,
        C,
    >,
>;

#[allow(clippy::type_complexity)]
impl<
        S,
        I,
        MinXS,
        MinXF: RecordSelection,
        MinYS,
        MinYF: RecordSelection,
        MinZS,
        MinZF: RecordSelection,
        MaxXS,
        MaxXF: RecordSelection,
        MaxYS,
        MaxYF: RecordSelection,
        MaxZS,
        MaxZF: RecordSelection,
        C,
    >
    WorthQueryBox3DIndex<
        S,
        I,
        MinXS,
        MinXF,
        MinYS,
        MinYF,
        MinZS,
        MinZF,
        MaxXS,
        MaxXF,
        MaxYS,
        MaxYF,
        MaxZS,
        MaxZF,
        C,
    >
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    /// Half-open `[min_x, max_x) x [min_y, max_y) x [min_z, max_z)` box index.
    pub const fn half_open(
        min: (
            FieldRef<S, I, MinXS, MinXF>,
            FieldRef<S, I, MinYS, MinYF>,
            FieldRef<S, I, MinZS, MinZF>,
        ),
        max: (
            FieldRef<S, I, MaxXS, MaxXF>,
            FieldRef<S, I, MaxYS, MaxYF>,
            FieldRef<S, I, MaxZS, MaxZF>,
        ),
    ) -> Self {
        Self::new_with_inclusion(min, max, WorthQueryRangeInclusion::HalfOpen)
    }

    /// Closed `[min_x, max_x] x [min_y, max_y] x [min_z, max_z]` box index.
    pub const fn closed(
        min: (
            FieldRef<S, I, MinXS, MinXF>,
            FieldRef<S, I, MinYS, MinYF>,
            FieldRef<S, I, MinZS, MinZF>,
        ),
        max: (
            FieldRef<S, I, MaxXS, MaxXF>,
            FieldRef<S, I, MaxYS, MaxYF>,
            FieldRef<S, I, MaxZS, MaxZF>,
        ),
    ) -> Self {
        Self::new_with_inclusion(min, max, WorthQueryRangeInclusion::Closed)
    }

    /// 3D box index with explicit [`WorthQueryRangeInclusion`].
    pub const fn new_with_inclusion(
        min: (
            FieldRef<S, I, MinXS, MinXF>,
            FieldRef<S, I, MinYS, MinYF>,
            FieldRef<S, I, MinZS, MinZF>,
        ),
        max: (
            FieldRef<S, I, MaxXS, MaxXF>,
            FieldRef<S, I, MaxYS, MaxYF>,
            FieldRef<S, I, MaxZS, MaxZF>,
        ),
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
    [
        (MinXS, MinXF, min_x),
        (MinYS, MinYF, min_y),
        (MinZS, MinZF, min_z)
    ],
    [
        (MaxXS, MaxXF, max_x),
        (MaxYS, MaxYF, max_y),
        (MaxZS, MaxZF, max_z)
    ]
);
