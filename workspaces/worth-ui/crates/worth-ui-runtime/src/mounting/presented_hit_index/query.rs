use super::*;
use crate::mounting::presentation::UiPlatformPoint;
use crate::mounting::spatial_index::{UiMountedSpatialBudget, UiMountedSpatialQueryDenial};
use worth_ui_host_contract::UiMountedCoordinateSpace;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentedHitQueryDenial {
    IncompatibleCoordinateSpace {
        row: UiMountedCoordinateSpace,
        work: UiHitTestSpatialWork,
    },
    InvalidPoint {
        work: UiHitTestSpatialWork,
    },
    NodeBudget {
        work: UiHitTestSpatialWork,
    },
    CandidateBudget {
        work: UiHitTestSpatialWork,
    },
}

pub(in crate::mounting) struct UiPresentedHitQuery {
    pub(in crate::mounting) rows: Vec<UiPresentedHitTestRow>,
    pub(in crate::mounting) work: UiHitTestSpatialWork,
}

impl UiPresentedHitQueryDenial {
    pub(crate) const fn work(self) -> UiHitTestSpatialWork {
        match self {
            Self::IncompatibleCoordinateSpace { work, .. }
            | Self::InvalidPoint { work }
            | Self::NodeBudget { work }
            | Self::CandidateBudget { work } => work,
        }
    }
}

impl UiPresentedHitIndex {
    /// The rows whose presented hit region holds a platform point.
    pub(in crate::mounting) fn at_point(
        &self,
        binding: UiSurfaceBindingGeneration,
        point: UiPlatformPoint,
        budget: UiMountedSpatialBudget,
    ) -> Result<UiPresentedHitQuery, UiPresentedHitQueryDenial> {
        self.at_index_point(binding, point.index_point(), budget)
    }

    /// The same query at a raw index point. Only the index reaches it, so its
    /// tests can probe precision and nonfinite points no platform reports.
    pub(super) fn at_index_point(
        &self,
        binding: UiSurfaceBindingGeneration,
        point: [f64; 2],
        budget: UiMountedSpatialBudget,
    ) -> Result<UiPresentedHitQuery, UiPresentedHitQueryDenial> {
        let mut work = UiHitTestSpatialWork::default();
        for space in [
            UiMountedCoordinateSpace::Window,
            UiMountedCoordinateSpace::GraphNodeLocal,
            UiMountedCoordinateSpace::HostSurface,
            UiMountedCoordinateSpace::PortalLayer,
        ] {
            let (partition, probes) = self.partitions.get_with_probes(&(binding, space as u8));
            work.map_key_probes += probes;
            if partition.is_some_and(|partition| partition.visible_rows != 0) {
                return Err(UiPresentedHitQueryDenial::IncompatibleCoordinateSpace {
                    row: space,
                    work,
                });
            }
        }
        if point.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(UiPresentedHitQueryDenial::InvalidPoint { work });
        }
        let (partition, probes) = self
            .partitions
            .get_with_probes(&(binding, UiMountedCoordinateSpace::Viewport as u8));
        work.map_key_probes += probes;
        let Some(partition) = partition else {
            return Ok(UiPresentedHitQuery {
                rows: Vec::new(),
                work,
            });
        };
        let query = partition
            .tree
            .at_point(point, budget)
            .map_err(|denial| match denial {
                UiMountedSpatialQueryDenial::InvalidGeometry => {
                    UiPresentedHitQueryDenial::InvalidPoint { work }
                }
                UiMountedSpatialQueryDenial::NodeBudget { work: spatial } => {
                    work.merge(spatial);
                    UiPresentedHitQueryDenial::NodeBudget { work }
                }
                UiMountedSpatialQueryDenial::CandidateBudget { work: spatial } => {
                    work.merge(spatial);
                    UiPresentedHitQueryDenial::CandidateBudget { work }
                }
            })?;
        work.merge(query.work);
        let rows = query
            .instances
            .into_iter()
            .map(|instance| {
                let (record, probes) = self.rows.get_with_probes(&instance);
                work.map_key_probes += probes;
                record
                    .and_then(|record| record.effective)
                    .expect("spatial membership has one effective row")
            })
            .collect();
        Ok(UiPresentedHitQuery { rows, work })
    }

    /// Every row a point on `binding` can land on, found by a full scan
    /// rather than the spatial index.
    pub(in crate::mounting) fn rows_on(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> impl Iterator<Item = UiPresentedHitTestRow> + '_ {
        self.rows
            .iter()
            .filter_map(|(_, record)| record.effective)
            .filter(move |row| row.mounted().binding() == binding)
    }
}
