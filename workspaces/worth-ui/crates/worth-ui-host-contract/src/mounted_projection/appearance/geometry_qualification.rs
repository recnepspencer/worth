use super::{UiAppearanceLogicalLength, UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT};

/// Maximum number of independently qualified device scales in one staged profile.
pub const UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostAppearanceGeometryQualificationBasis {
    AnalyticSignedDistancePixelCenter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostAppearanceScaleGeometryQualification {
    device_scale_milli: u32,
    anti_alias_fringe_physical_pixels: u32,
    anti_alias_fringe_logical_subpixels: UiAppearanceLogicalLength,
    basis: UiHostAppearanceGeometryQualificationBasis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostAppearanceGeometryQualificationDenial {
    Empty,
    CapacityExceeded,
    NonCanonicalOrder,
    DuplicateDeviceScale(u32),
    ZeroDeviceScale,
    ZeroPhysicalFringe,
    LogicalEnclosureOutOfRange,
    NonCanonicalLogicalEnclosure {
        device_scale_milli: u32,
        expected_subpixels: u32,
        actual_subpixels: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostAppearanceScaleDenial {
    ZeroDeviceScale,
    UnsupportedScale(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHostAppearanceGeometryQualification {
    rows: Box<[UiHostAppearanceScaleGeometryQualification]>,
}

impl UiHostAppearanceScaleGeometryQualification {
    pub const fn new(
        device_scale_milli: u32,
        anti_alias_fringe_physical_pixels: u32,
        anti_alias_fringe_logical_subpixels: UiAppearanceLogicalLength,
        basis: UiHostAppearanceGeometryQualificationBasis,
    ) -> Self {
        Self {
            device_scale_milli,
            anti_alias_fringe_physical_pixels,
            anti_alias_fringe_logical_subpixels,
            basis,
        }
    }

    pub const fn device_scale_milli(self) -> u32 {
        self.device_scale_milli
    }

    pub const fn anti_alias_fringe_physical_pixels(self) -> u32 {
        self.anti_alias_fringe_physical_pixels
    }

    pub const fn anti_alias_fringe_logical_subpixels(self) -> UiAppearanceLogicalLength {
        self.anti_alias_fringe_logical_subpixels
    }

    pub const fn basis(self) -> UiHostAppearanceGeometryQualificationBasis {
        self.basis
    }
}

impl UiHostAppearanceGeometryQualification {
    pub fn admit(
        rows: impl IntoIterator<Item = UiHostAppearanceScaleGeometryQualification>,
    ) -> Result<Self, UiHostAppearanceGeometryQualificationDenial> {
        let mut admitted_rows = Vec::with_capacity(UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY + 1);
        for row in rows
            .into_iter()
            .take(UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY + 1)
        {
            admitted_rows.push(row);
        }
        let rows = admitted_rows;
        if rows.is_empty() {
            return Err(UiHostAppearanceGeometryQualificationDenial::Empty);
        }
        if rows.len() > UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY {
            return Err(UiHostAppearanceGeometryQualificationDenial::CapacityExceeded);
        }
        for pair in rows.windows(2) {
            if pair[0].device_scale_milli == pair[1].device_scale_milli {
                return Err(
                    UiHostAppearanceGeometryQualificationDenial::DuplicateDeviceScale(
                        pair[0].device_scale_milli,
                    ),
                );
            }
            if pair[0].device_scale_milli > pair[1].device_scale_milli {
                return Err(UiHostAppearanceGeometryQualificationDenial::NonCanonicalOrder);
            }
        }
        for row in &rows {
            validate_row(row)?;
        }
        Ok(Self {
            rows: rows.into_boxed_slice(),
        })
    }

    pub fn rows(&self) -> &[UiHostAppearanceScaleGeometryQualification] {
        &self.rows
    }

    pub fn row_for_scale(
        &self,
        device_scale_milli: u32,
    ) -> Result<UiHostAppearanceScaleGeometryQualification, UiHostAppearanceScaleDenial> {
        if device_scale_milli == 0 {
            return Err(UiHostAppearanceScaleDenial::ZeroDeviceScale);
        }
        self.rows
            .iter()
            .find(|row| row.device_scale_milli == device_scale_milli)
            .copied()
            .ok_or(UiHostAppearanceScaleDenial::UnsupportedScale(
                device_scale_milli,
            ))
    }

    pub(crate) fn append_canonical_encoding(
        &self,
        digest: &mut crate::runtime::WorthUiHostCapabilityDigest,
    ) {
        digest.update_u64(self.rows.len() as u64);
        for row in &self.rows {
            digest.update_u64(u64::from(row.device_scale_milli));
            digest.update_u64(u64::from(row.anti_alias_fringe_physical_pixels));
            digest.update_u64(u64::from(
                row.anti_alias_fringe_logical_subpixels.subpixels(),
            ));
            digest.update_byte(match row.basis {
                UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter => 1,
            });
        }
    }
}

fn validate_row(
    row: &UiHostAppearanceScaleGeometryQualification,
) -> Result<(), UiHostAppearanceGeometryQualificationDenial> {
    if row.device_scale_milli == 0 {
        return Err(UiHostAppearanceGeometryQualificationDenial::ZeroDeviceScale);
    }
    if row.anti_alias_fringe_physical_pixels == 0 {
        return Err(UiHostAppearanceGeometryQualificationDenial::ZeroPhysicalFringe);
    }
    let numerator = u128::from(row.anti_alias_fringe_physical_pixels)
        .checked_mul(u128::from(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT))
        .and_then(|value| value.checked_mul(1_000))
        .ok_or(UiHostAppearanceGeometryQualificationDenial::LogicalEnclosureOutOfRange)?;
    let denominator = u128::from(row.device_scale_milli);
    let expected = numerator
        .checked_add(denominator - 1)
        .ok_or(UiHostAppearanceGeometryQualificationDenial::LogicalEnclosureOutOfRange)?
        / denominator;
    let expected = u32::try_from(expected)
        .map_err(|_| UiHostAppearanceGeometryQualificationDenial::LogicalEnclosureOutOfRange)?;
    let actual = row.anti_alias_fringe_logical_subpixels.subpixels();
    if actual != expected {
        return Err(
            UiHostAppearanceGeometryQualificationDenial::NonCanonicalLogicalEnclosure {
                device_scale_milli: row.device_scale_milli,
                expected_subpixels: expected,
                actual_subpixels: actual,
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(scale: u32, logical: u32) -> UiHostAppearanceScaleGeometryQualification {
        UiHostAppearanceScaleGeometryQualification::new(
            scale,
            1,
            UiAppearanceLogicalLength::new(logical as i32).unwrap(),
            UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
        )
    }

    #[test]
    fn qualification_keeps_exact_integer_enclosures_and_denies_unknown_scale() {
        let qualification = UiHostAppearanceGeometryQualification::admit([
            row(1_000, 1_000),
            row(1_250, 800),
            row(1_500, 667),
            row(2_000, 500),
        ])
        .unwrap();

        assert_eq!(
            qualification
                .row_for_scale(1_500)
                .unwrap()
                .anti_alias_fringe_logical_subpixels()
                .subpixels(),
            667
        );
        assert_eq!(
            qualification.row_for_scale(1_333),
            Err(UiHostAppearanceScaleDenial::UnsupportedScale(1_333))
        );
    }

    #[test]
    fn qualification_rejects_unsorted_duplicate_and_noncanonical_rows() {
        assert_eq!(
            UiHostAppearanceGeometryQualification::admit([row(1_250, 800), row(1_000, 1_000)]),
            Err(UiHostAppearanceGeometryQualificationDenial::NonCanonicalOrder)
        );
        assert_eq!(
            UiHostAppearanceGeometryQualification::admit([row(1_000, 1_000), row(1_000, 1_000)]),
            Err(UiHostAppearanceGeometryQualificationDenial::DuplicateDeviceScale(1_000))
        );
        assert_eq!(
            UiHostAppearanceGeometryQualification::admit([row(1_250, 801)]),
            Err(
                UiHostAppearanceGeometryQualificationDenial::NonCanonicalLogicalEnclosure {
                    device_scale_milli: 1_250,
                    expected_subpixels: 800,
                    actual_subpixels: 801,
                }
            )
        );
    }

    #[test]
    fn qualification_stops_at_capacity_plus_one_without_draining_the_iterator() {
        struct BoundedRows {
            yielded: usize,
        }

        impl Iterator for BoundedRows {
            type Item = UiHostAppearanceScaleGeometryQualification;

            fn next(&mut self) -> Option<Self::Item> {
                if self.yielded == UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY + 1 {
                    panic!("geometry admission drained beyond capacity plus one");
                }
                self.yielded += 1;
                Some(row(1_000, 1_000))
            }
        }

        assert_eq!(
            UiHostAppearanceGeometryQualification::admit(BoundedRows { yielded: 0 }),
            Err(UiHostAppearanceGeometryQualificationDenial::CapacityExceeded)
        );
    }
}
