//! Damage regions gathered into a bounded set of clears.
//!
//! Every region costs one ordered replay query, and a scrolled text raster
//! emits one region each, so a moving list pays that query a thousand times
//! over for a handful of operations apiece. Regions are therefore gathered
//! into grid cells once their number passes what separate queries carry well.
//!
//! A cell replays every command its area covers, so gathering moves the same
//! drawing into fewer queries instead of adding drawing. It clears more than
//! the regions strictly asked for, which stays correct because everything the
//! wider clear erases is replayed over it in the same pass.
use super::super::raster::raster_physical_bounds;
use super::super::RasterRect;

/// Region counts above this are gathered rather than replayed one at a time.
const GATHERED_ABOVE: usize = 64;
/// Cells along each surface axis when gathering.
const CELLS: u32 = 8;

pub(super) fn gather(regions: Vec<RasterRect>, extent: [u32; 2]) -> Vec<RasterRect> {
    if regions.len() <= GATHERED_ABOVE {
        return regions;
    }
    let mut cells = [None; CELLS as usize * CELLS as usize];
    for region in &regions {
        let bounds = physical_edges(*region);
        let cell = &mut cells[cell_of(bounds, extent)];
        *cell = Some(match *cell {
            Some(gathered) => union(gathered, bounds),
            None => bounds,
        });
    }
    let gathered = cells
        .into_iter()
        .flatten()
        .filter_map(|bounds| raster_physical_bounds(bounds, extent))
        .collect::<Vec<_>>();
    // A surface too small for the grid, or bounds the extent refuses, would
    // leave damage unpainted. The separate regions are always paintable.
    if gathered.is_empty() {
        return regions;
    }
    gathered
}

/// `[left, top, right, bottom]` in physical pixels.
fn physical_edges(region: RasterRect) -> [u32; 4] {
    let [left, top, width, height] = region.physical_bounds();
    [
        left as u32,
        top as u32,
        (left + width) as u32,
        (top + height) as u32,
    ]
}

/// The cell holding a region's centre, so a region joins one cell only.
fn cell_of(bounds: [u32; 4], extent: [u32; 2]) -> usize {
    let column = axis_cell((bounds[0] + bounds[2]) / 2, extent[0]);
    let row = axis_cell((bounds[1] + bounds[3]) / 2, extent[1]);
    (row * CELLS + column) as usize
}

fn axis_cell(centre: u32, extent: u32) -> u32 {
    (centre * CELLS / extent.max(1)).min(CELLS - 1)
}

fn union(gathered: [u32; 4], bounds: [u32; 4]) -> [u32; 4] {
    [
        gathered[0].min(bounds[0]),
        gathered[1].min(bounds[1]),
        gathered[2].max(bounds[2]),
        gathered[3].max(bounds[3]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTENT: [u32; 2] = [2_304, 1_536];

    fn region(bounds: [u32; 4]) -> RasterRect {
        raster_physical_bounds(bounds, EXTENT).unwrap()
    }

    #[test]
    fn region_counts_a_replay_carries_are_left_alone() {
        let regions = (0..GATHERED_ABOVE)
            .map(|index| region([0, index as u32 * 20, 100, index as u32 * 20 + 10]))
            .collect::<Vec<_>>();
        let gathered = gather(regions.clone(), EXTENT);
        assert_eq!(gathered.len(), regions.len());
    }

    #[test]
    fn gathering_covers_every_region_it_replaces() {
        let regions = (0..600)
            .map(|index| {
                let top = index * 2;
                region([index % 900, top, index % 900 + 40, top + 12])
            })
            .collect::<Vec<_>>();
        let gathered = gather(regions.clone(), EXTENT);
        assert!(
            gathered.len() <= (CELLS * CELLS) as usize,
            "gathering bounds the replay count"
        );
        assert!(gathered.len() < regions.len(), "gathering removes queries");
        for region in &regions {
            let bounds = physical_edges(*region);
            assert!(
                gathered
                    .iter()
                    .any(|cell| covers(physical_edges(*cell), bounds)),
                "every damaged pixel stays inside a clear: {bounds:?}"
            );
        }
    }

    fn covers(outer: [u32; 4], inner: [u32; 4]) -> bool {
        outer[0] <= inner[0] && outer[1] <= inner[1] && outer[2] >= inner[2] && outer[3] >= inner[3]
    }
}
