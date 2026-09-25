use crate::capability::MosaicTrack;

/// One span of a layout axis, such as an allocated track: its offset from
/// the axis start and its extent, in logical points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMosaicAxisSpan {
    pub(crate) start: f32,
    pub(crate) extent: f32,
}

/// Allocates one layout axis.
///
/// Fixed tracks keep their extent. The space left after fixed tracks and gaps
/// is shared among flexible tracks by weight, and each share is clamped to
/// its bounds. When clamping adds space overall, the tracks raised to their
/// minimum are held there; when it removes space, the tracks lowered to their
/// maximum are held there. The rest is shared again among the others until no
/// share violates a bound. When the space cannot hold every minimum, the
/// tracks keep their minimums and overflow the axis; no extent is negative.
pub(crate) fn allocate_axis(
    tracks: &[MosaicTrack],
    gap: f32,
    available: f32,
) -> Vec<UiMosaicAxisSpan> {
    let gaps = gap * tracks.len().saturating_sub(1) as f32;
    let mut extents = tracks
        .iter()
        .map(|track| f32::from(track.base_logical_points()))
        .collect::<Vec<_>>();
    let fixed = tracks
        .iter()
        .zip(&extents)
        .filter(|(track, _)| track.weight().is_none())
        .map(|(_, extent)| extent)
        .sum::<f32>();
    let mut flexible_space = available - gaps - fixed;
    let mut open = (0..tracks.len())
        .filter(|index| tracks[*index].weight().is_some())
        .collect::<Vec<_>>();
    while !open.is_empty() {
        let weights = open.iter().map(|index| weight(tracks[*index])).sum::<f32>();
        let per_weight = flexible_space / weights;
        let shares = open
            .iter()
            .map(|index| {
                let track = tracks[*index];
                let share = per_weight * weight(track);
                let maximum = track.max_logical_points().map_or(f32::INFINITY, f32::from);
                let minimum = f32::from(track.base_logical_points());
                (*index, share, share.min(maximum).max(minimum))
            })
            .collect::<Vec<_>>();
        let violation = shares
            .iter()
            .map(|(_, share, clamped)| clamped - share)
            .sum::<f32>();
        for (index, share, clamped) in shares {
            let held = if violation > 0.0 {
                clamped > share
            } else if violation < 0.0 {
                clamped < share
            } else {
                true
            };
            if held {
                extents[index] = clamped;
                flexible_space -= clamped;
                open.retain(|open| *open != index);
            }
        }
    }
    let mut start = 0.0;
    extents
        .into_iter()
        .map(|extent| {
            let track = UiMosaicAxisSpan { start, extent };
            start += extent + gap;
            track
        })
        .collect()
}

fn weight(track: MosaicTrack) -> f32 {
    track.weight().map_or(0.0, f32::from)
}

#[cfg(test)]
mod tests {
    use super::{allocate_axis, UiMosaicAxisSpan};
    use crate::capability::MosaicTrack;

    fn extents(tracks: &[UiMosaicAxisSpan]) -> Vec<f32> {
        tracks.iter().map(|track| track.extent).collect()
    }

    #[test]
    fn weights_share_the_space_after_fixed_tracks_and_gaps() {
        let tracks = [
            MosaicTrack::fixed(236).unwrap(),
            MosaicTrack::flex(2, 0).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ];
        let allocated = allocate_axis(&tracks, 20.0, 1176.0);
        assert_eq!(extents(&allocated), [236.0, 600.0, 300.0]);
        assert_eq!(allocated[1].start, 256.0);
        assert_eq!(allocated[2].start, 876.0);
    }

    #[test]
    fn a_share_below_its_minimum_is_held_and_the_rest_is_shared_again() {
        let tracks = [
            MosaicTrack::flex(2, 480).unwrap(),
            MosaicTrack::flex(1, 320).unwrap(),
        ];
        // An even 2:1 split of 880 gives the second track 293.3.
        assert_eq!(extents(&allocate_axis(&tracks, 0.0, 880.0)), [560.0, 320.0]);
        assert_eq!(
            extents(&allocate_axis(&tracks, 0.0, 1200.0)),
            [800.0, 400.0]
        );
    }

    #[test]
    fn a_share_above_its_maximum_is_held_and_the_rest_is_shared_again() {
        let tracks = [
            MosaicTrack::bounded_flex(1, 0, 100).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ];
        assert_eq!(
            extents(&allocate_axis(&tracks, 0.0, 600.0)),
            [100.0, 250.0, 250.0]
        );
    }

    #[test]
    fn space_a_maximum_frees_returns_to_a_track_held_at_its_minimum() {
        let tracks = [
            MosaicTrack::flex(1, 60).unwrap(),
            MosaicTrack::bounded_flex(1, 0, 1).unwrap(),
        ];
        // An even split of 100 raises the first track to 60 and lowers the
        // second to 1. The cap removes more than the minimum adds, so the cap
        // is held first and the first track takes the other 99.
        assert_eq!(extents(&allocate_axis(&tracks, 0.0, 100.0)), [99.0, 1.0]);
    }

    #[test]
    fn insufficient_space_keeps_minimums_and_overflows_without_negative_extents() {
        let tracks = [
            MosaicTrack::fixed(236).unwrap(),
            MosaicTrack::flex(2, 480).unwrap(),
            MosaicTrack::flex(1, 320).unwrap(),
        ];
        let allocated = allocate_axis(&tracks, 24.0, 400.0);
        assert_eq!(extents(&allocated), [236.0, 480.0, 320.0]);
        assert_eq!(allocated[2].start, 236.0 + 24.0 + 480.0 + 24.0);
        let empty = allocate_axis(&[MosaicTrack::flex(1, 0).unwrap()], 0.0, -40.0);
        assert_eq!(extents(&empty), [0.0]);
    }
}
