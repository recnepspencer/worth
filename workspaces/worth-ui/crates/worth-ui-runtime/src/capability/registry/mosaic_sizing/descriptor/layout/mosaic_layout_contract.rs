use super::{MosaicLayoutDenial, MosaicTrack};
use crate::capability::ComponentId;

/// A container's declared track layout: column and row tracks, the gaps
/// between them, the padding around them, and the member component each cell
/// holds.
///
/// A member places itself within its cell through its own
/// `ComponentAllocationMeasurementContract::LayoutCell` region; the container
/// decides only which cell that is. A layout with one column and one row
/// frames every member in the container's own box.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicLayoutContract {
    columns: Box<[MosaicTrack]>,
    rows: Box<[MosaicTrack]>,
    column_gap_logical_points: u16,
    row_gap_logical_points: u16,
    inline_padding_logical_points: u16,
    block_padding_logical_points: u16,
    members: Vec<MosaicLayoutMember>,
}

/// One member component and the cells it spans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicLayoutMember {
    component: ComponentId,
    cell: MosaicLayoutCell,
}

/// A half-open span of column and row tracks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MosaicLayoutCell {
    column: u16,
    row: u16,
    column_span: u16,
    row_span: u16,
}

impl MosaicLayoutCell {
    pub const fn at(column: u16, row: u16) -> Self {
        Self {
            column,
            row,
            column_span: 1,
            row_span: 1,
        }
    }

    pub const fn spanning(
        column: u16,
        row: u16,
        column_span: u16,
        row_span: u16,
    ) -> Result<Self, MosaicLayoutDenial> {
        if column_span == 0 || row_span == 0 {
            return Err(MosaicLayoutDenial::EmptyCell);
        }
        Ok(Self {
            column,
            row,
            column_span,
            row_span,
        })
    }

    pub const fn column(self) -> u16 {
        self.column
    }

    pub const fn row(self) -> u16 {
        self.row
    }

    pub const fn column_span(self) -> u16 {
        self.column_span
    }

    pub const fn row_span(self) -> u16 {
        self.row_span
    }
}

impl MosaicLayoutContract {
    pub fn grid(
        columns: impl IntoIterator<Item = MosaicTrack>,
        rows: impl IntoIterator<Item = MosaicTrack>,
    ) -> Result<Self, MosaicLayoutDenial> {
        let columns = columns.into_iter().collect::<Box<[_]>>();
        let rows = rows.into_iter().collect::<Box<[_]>>();
        if columns.is_empty() || rows.is_empty() {
            return Err(MosaicLayoutDenial::NoTracks);
        }
        if columns.len() > usize::from(u16::MAX) || rows.len() > usize::from(u16::MAX) {
            return Err(MosaicLayoutDenial::TooManyTracks);
        }
        Ok(Self {
            columns,
            rows,
            column_gap_logical_points: 0,
            row_gap_logical_points: 0,
            inline_padding_logical_points: 0,
            block_padding_logical_points: 0,
            members: Vec::new(),
        })
    }

    /// Columns in one row that fills the container's height.
    pub fn columns(
        columns: impl IntoIterator<Item = MosaicTrack>,
    ) -> Result<Self, MosaicLayoutDenial> {
        Self::grid(columns, [MosaicTrack::flex(1, 0)?])
    }

    /// Rows in one column that fills the container's width.
    pub fn rows(rows: impl IntoIterator<Item = MosaicTrack>) -> Result<Self, MosaicLayoutDenial> {
        Self::grid([MosaicTrack::flex(1, 0)?], rows)
    }

    /// One cell that frames every member in the container's own box.
    pub fn frame() -> Result<Self, MosaicLayoutDenial> {
        Self::grid([MosaicTrack::flex(1, 0)?], [MosaicTrack::flex(1, 0)?])
    }

    pub fn with_gaps(mut self, column_gap: u16, row_gap: u16) -> Self {
        self.column_gap_logical_points = column_gap;
        self.row_gap_logical_points = row_gap;
        self
    }

    /// Keeps the tracks `inline` points from the container's left and right
    /// edges and `block` points from its top and bottom.
    pub fn with_padding(mut self, inline: u16, block: u16) -> Self {
        self.inline_padding_logical_points = inline;
        self.block_padding_logical_points = block;
        self
    }

    /// Adds one member to one cell. A component may be a member once, and the
    /// cell must lie within the declared tracks.
    pub fn with_member(
        mut self,
        component: ComponentId,
        cell: MosaicLayoutCell,
    ) -> Result<Self, MosaicLayoutDenial> {
        let column_end = usize::from(cell.column) + usize::from(cell.column_span);
        let row_end = usize::from(cell.row) + usize::from(cell.row_span);
        if column_end > self.columns.len() || row_end > self.rows.len() {
            return Err(MosaicLayoutDenial::CellOutsideTracks);
        }
        if self.member_cell(&component).is_some() {
            return Err(MosaicLayoutDenial::DuplicateMember);
        }
        self.members.push(MosaicLayoutMember { component, cell });
        Ok(self)
    }

    pub fn column_tracks(&self) -> &[MosaicTrack] {
        &self.columns
    }

    pub fn row_tracks(&self) -> &[MosaicTrack] {
        &self.rows
    }

    pub const fn column_gap_logical_points(&self) -> u16 {
        self.column_gap_logical_points
    }

    pub const fn row_gap_logical_points(&self) -> u16 {
        self.row_gap_logical_points
    }

    pub const fn inline_padding_logical_points(&self) -> u16 {
        self.inline_padding_logical_points
    }

    pub const fn block_padding_logical_points(&self) -> u16 {
        self.block_padding_logical_points
    }

    /// The narrowest box that holds every column at its minimum, with the
    /// gaps between them and the padding on both sides.
    pub fn minimum_width_logical_points(&self) -> u32 {
        minimum_extent(
            &self.columns,
            self.column_gap_logical_points,
            self.inline_padding_logical_points,
        )
    }

    /// The shortest box that holds every row at its minimum, with the gaps
    /// between them and the padding above and below.
    pub fn minimum_height_logical_points(&self) -> u32 {
        minimum_extent(
            &self.rows,
            self.row_gap_logical_points,
            self.block_padding_logical_points,
        )
    }

    pub fn members(&self) -> impl Iterator<Item = (&ComponentId, MosaicLayoutCell)> {
        self.members
            .iter()
            .map(|member| (&member.component, member.cell))
    }

    pub fn member_cell(&self, component: &ComponentId) -> Option<MosaicLayoutCell> {
        self.members
            .iter()
            .find(|member| &member.component == component)
            .map(|member| member.cell)
    }

    pub(crate) fn digest_basis(&self) -> String {
        let tracks = |tracks: &[MosaicTrack]| {
            tracks
                .iter()
                .map(|track| track.digest_basis())
                .collect::<Vec<_>>()
                .join(",")
        };
        let members = self
            .members
            .iter()
            .map(|member| {
                format!(
                    "{}@{}:{}:{}:{}",
                    member.component.as_str(),
                    member.cell.column,
                    member.cell.row,
                    member.cell.column_span,
                    member.cell.row_span,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "grid:[{}]:[{}]:{}:{}:{}:{}:[{members}]",
            tracks(&self.columns),
            tracks(&self.rows),
            self.column_gap_logical_points,
            self.row_gap_logical_points,
            self.inline_padding_logical_points,
            self.block_padding_logical_points,
        )
    }
}

/// Every track's minimum, the gaps between the tracks, and the padding on
/// both sides of them.
fn minimum_extent(tracks: &[MosaicTrack], gap: u16, padding: u16) -> u32 {
    let tracks_extent = tracks
        .iter()
        .map(|track| u32::from(track.base_logical_points()))
        .sum::<u32>();
    let gaps = u32::from(gap) * (tracks.len() as u32).saturating_sub(1);
    tracks_extent + gaps + 2 * u32::from(padding)
}

#[cfg(test)]
mod tests {
    use super::{MosaicLayoutCell, MosaicLayoutContract, MosaicLayoutDenial, MosaicTrack};
    use crate::capability::ComponentId;

    fn id(value: &str) -> ComponentId {
        ComponentId::new(value).unwrap()
    }

    #[test]
    fn members_must_lie_within_tracks_and_appear_once() {
        let layout = MosaicLayoutContract::columns([
            MosaicTrack::flex(2, 480).unwrap(),
            MosaicTrack::flex(1, 320).unwrap(),
        ])
        .unwrap()
        .with_gaps(20, 20);
        assert_eq!(
            layout
                .clone()
                .with_member(id("demo.component.chart"), MosaicLayoutCell::at(2, 0)),
            Err(MosaicLayoutDenial::CellOutsideTracks)
        );
        let layout = layout
            .with_member(id("demo.component.chart"), MosaicLayoutCell::at(0, 0))
            .unwrap();
        assert_eq!(
            layout
                .clone()
                .with_member(id("demo.component.chart"), MosaicLayoutCell::at(1, 0)),
            Err(MosaicLayoutDenial::DuplicateMember)
        );
        assert_eq!(
            layout.member_cell(&id("demo.component.chart")),
            Some(MosaicLayoutCell::at(0, 0))
        );
        assert_eq!(
            MosaicLayoutCell::spanning(0, 0, 0, 1),
            Err(MosaicLayoutDenial::EmptyCell)
        );
        assert_eq!(
            MosaicLayoutContract::grid([], [MosaicTrack::fixed(1).unwrap()]),
            Err(MosaicLayoutDenial::NoTracks)
        );
    }

    #[test]
    fn minimum_extent_holds_every_minimum_with_its_gaps_and_padding() {
        let layout = MosaicLayoutContract::grid(
            [
                MosaicTrack::flex(2, 480).unwrap(),
                MosaicTrack::flex(1, 320).unwrap(),
            ],
            [
                MosaicTrack::fixed(66).unwrap(),
                MosaicTrack::flex(1, 348).unwrap(),
            ],
        )
        .unwrap()
        .with_gaps(20, 10)
        .with_padding(24, 12);
        assert_eq!(layout.minimum_width_logical_points(), 480 + 20 + 320 + 48);
        assert_eq!(layout.minimum_height_logical_points(), 66 + 10 + 348 + 24);
        assert_eq!(
            MosaicLayoutContract::frame()
                .unwrap()
                .minimum_width_logical_points(),
            0
        );
    }
}
