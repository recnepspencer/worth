use super::MosaicLayoutDenial;

/// One declared track of a Mosaic layout axis, in logical points.
///
/// Fixed tracks keep their extent. Flexible tracks share the remaining space
/// by weight within their bounds; each round, the tracks whose bound corrects
/// the larger total violation are held at it and the rest is shared again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MosaicTrack {
    kind: MosaicTrackKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MosaicTrackKind {
    Fixed {
        extent_logical_points: u16,
    },
    Flexible {
        weight: u16,
        min_logical_points: u16,
        max_logical_points: Option<u16>,
    },
}

impl MosaicTrack {
    pub const fn fixed(extent_logical_points: u16) -> Result<Self, MosaicLayoutDenial> {
        if extent_logical_points == 0 {
            return Err(MosaicLayoutDenial::EmptyTrack);
        }
        Ok(Self {
            kind: MosaicTrackKind::Fixed {
                extent_logical_points,
            },
        })
    }

    pub const fn flex(weight: u16, min_logical_points: u16) -> Result<Self, MosaicLayoutDenial> {
        if weight == 0 {
            return Err(MosaicLayoutDenial::ZeroWeight);
        }
        Ok(Self {
            kind: MosaicTrackKind::Flexible {
                weight,
                min_logical_points,
                max_logical_points: None,
            },
        })
    }

    pub const fn bounded_flex(
        weight: u16,
        min_logical_points: u16,
        max_logical_points: u16,
    ) -> Result<Self, MosaicLayoutDenial> {
        if weight == 0 {
            return Err(MosaicLayoutDenial::ZeroWeight);
        }
        if max_logical_points == 0 || min_logical_points > max_logical_points {
            return Err(MosaicLayoutDenial::InconsistentBounds);
        }
        Ok(Self {
            kind: MosaicTrackKind::Flexible {
                weight,
                min_logical_points,
                max_logical_points: Some(max_logical_points),
            },
        })
    }

    /// The extent this track keeps before any remaining space is shared: a
    /// fixed extent or a flexible minimum.
    pub const fn base_logical_points(self) -> u16 {
        match self.kind {
            MosaicTrackKind::Fixed {
                extent_logical_points,
            } => extent_logical_points,
            MosaicTrackKind::Flexible {
                min_logical_points, ..
            } => min_logical_points,
        }
    }

    /// The nonzero share weight of a flexible track; `None` for a fixed one.
    pub const fn weight(self) -> Option<u16> {
        match self.kind {
            MosaicTrackKind::Fixed { .. } => None,
            MosaicTrackKind::Flexible { weight, .. } => Some(weight),
        }
    }

    /// The largest extent a bounded flexible track may take.
    pub const fn max_logical_points(self) -> Option<u16> {
        match self.kind {
            MosaicTrackKind::Fixed { .. } => None,
            MosaicTrackKind::Flexible {
                max_logical_points, ..
            } => max_logical_points,
        }
    }

    pub(crate) fn digest_basis(self) -> String {
        match self.kind {
            MosaicTrackKind::Fixed {
                extent_logical_points,
            } => format!("fixed:{extent_logical_points}"),
            MosaicTrackKind::Flexible {
                weight,
                min_logical_points,
                max_logical_points: None,
            } => format!("flex:{weight}:{min_logical_points}"),
            MosaicTrackKind::Flexible {
                weight,
                min_logical_points,
                max_logical_points: Some(max),
            } => format!("flex:{weight}:{min_logical_points}:{max}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MosaicLayoutDenial, MosaicTrack};

    #[test]
    fn tracks_reject_empty_extent_zero_weight_and_inverted_bounds() {
        assert_eq!(MosaicTrack::fixed(0), Err(MosaicLayoutDenial::EmptyTrack));
        assert_eq!(
            MosaicTrack::flex(0, 480),
            Err(MosaicLayoutDenial::ZeroWeight)
        );
        assert_eq!(
            MosaicTrack::bounded_flex(1, 480, 320),
            Err(MosaicLayoutDenial::InconsistentBounds)
        );
        assert_eq!(
            MosaicTrack::bounded_flex(2, 480, 960)
                .unwrap()
                .digest_basis(),
            "flex:2:480:960"
        );
        assert_eq!(MosaicTrack::fixed(236).unwrap().base_logical_points(), 236);
    }
}
