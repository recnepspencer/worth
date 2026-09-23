//! The trailing Owner / Environment / Duration / Status columns that give the
//! Recent activity list its inline overflow.
//!
//! Every band authors a real cell in every column, so the list's inline extent
//! is occupied content rather than empty space reserved for a test.
use super::super::{text, DashboardElement};
use super::rows::{row_band_top, PLATFORM_PULSE_ACTIVITY_ROW_COUNT};
use super::scrolled;

/// Block inset and extent of a trailing cell inside its row band.
const CELL_BLOCK_INSET: u16 = 3;
const CELL_BLOCK_EXTENT: u16 = 24;
const CELL_TEXT_SIZE: u16 = 13;

/// The number of trailing columns every row band authors.
const TRAILING_COLUMN_COUNT: usize = 4;

/// One trailing column of the Recent activity list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActivityColumn {
    Owner,
    Environment,
    Duration,
    Status,
}

impl ActivityColumn {
    const ALL: [Self; TRAILING_COLUMN_COUNT] =
        [Self::Owner, Self::Environment, Self::Duration, Self::Status];

    /// Inline offset of the column from the content origin.
    const fn content_local_x(self) -> u16 {
        match self {
            Self::Owner => 600,
            Self::Environment => 852,
            Self::Duration => 1_124,
            Self::Status => 1_316,
        }
    }

    const fn width(self) -> u16 {
        match self {
            Self::Owner => 220,
            Self::Environment => 240,
            Self::Duration => 160,
            Self::Status => 220,
        }
    }

    const fn color(self) -> &'static str {
        match self {
            Self::Owner | Self::Environment | Self::Duration => "secondary_text",
            Self::Status => "primary_text",
        }
    }
}

/// The four trailing cells one row band authors, in `ActivityColumn::ALL` order.
struct ActivityRowCells {
    band: u16,
    identities: [&'static str; TRAILING_COLUMN_COUNT],
    values: [&'static str; TRAILING_COLUMN_COUNT],
}

/// Binds one band's authored cell text to the identities that band owns.
macro_rules! activity_row_cells {
    ($band:literal, $owner:literal, $environment:literal, $duration:literal, $status:literal) => {
        ActivityRowCells {
            band: $band,
            identities: [
                concat!("activity_owner_", $band),
                concat!("activity_environment_", $band),
                concat!("activity_duration_", $band),
                concat!("activity_status_", $band),
            ],
            values: [$owner, $environment, $duration, $status],
        }
    };
}

const CELLS: [ActivityRowCells; PLATFORM_PULSE_ACTIVITY_ROW_COUNT] = [
    activity_row_cells!(0, "Platform", "production", "1m 12s", "Succeeded"),
    activity_row_cells!(1, "Search", "production", "18s", "Applied"),
    activity_row_cells!(2, "Edge", "production-eu", "6m 04s", "Investigating"),
    activity_row_cells!(3, "Observability", "production", "42s", "Resolved"),
    activity_row_cells!(4, "Release", "staging", "Not started", "Scheduled"),
    activity_row_cells!(5, "Edge", "production-us", "3m 27s", "Completed"),
    activity_row_cells!(6, "Identity", "production", "Not started", "Pending"),
    activity_row_cells!(7, "Payments", "production-us", "4m 10s", "Mitigated"),
    activity_row_cells!(8, "Release", "production-eu", "2m 55s", "Verified"),
    activity_row_cells!(9, "Growth", "canary", "9s", "Enabled"),
    activity_row_cells!(10, "Data", "analytics", "1h 06m", "Warning"),
    activity_row_cells!(11, "Storage", "production", "21m 38s", "Completed"),
];

pub(super) fn activity_columns() -> Vec<DashboardElement> {
    CELLS.iter().flat_map(band_cells).collect()
}

fn band_cells(band: &ActivityRowCells) -> Vec<DashboardElement> {
    let top = row_band_top(band.band) + CELL_BLOCK_INSET;
    ActivityColumn::ALL
        .into_iter()
        .enumerate()
        .map(|(ordinal, column)| {
            scrolled(text(
                band.identities[ordinal],
                band.values[ordinal],
                [
                    column.content_local_x(),
                    top,
                    column.width(),
                    CELL_BLOCK_EXTENT,
                ],
                CELL_TEXT_SIZE,
                false,
                column.color(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identity stem every cell in a column carries before its band. Stated
    /// here so the authored identities are checked against a second statement
    /// rather than against themselves.
    const fn identity_stem(column: ActivityColumn) -> &'static str {
        match column {
            ActivityColumn::Owner => "activity_owner_",
            ActivityColumn::Environment => "activity_environment_",
            ActivityColumn::Duration => "activity_duration_",
            ActivityColumn::Status => "activity_status_",
        }
    }

    #[test]
    fn every_authored_cell_identity_names_its_column_and_band() {
        for cells in &CELLS {
            for (ordinal, column) in ActivityColumn::ALL.into_iter().enumerate() {
                assert_eq!(
                    cells.identities[ordinal],
                    format!("{}{}", identity_stem(column), cells.band),
                );
            }
        }
    }

    #[test]
    fn the_trailing_columns_do_not_overlap_and_stay_ordered() {
        let mut previous_right = 0;
        for column in ActivityColumn::ALL {
            assert!(
                column.content_local_x() >= previous_right,
                "{column:?} starts before the previous column ends",
            );
            previous_right = column.content_local_x() + column.width();
        }
    }
}
