use crate::UiDslComponentReference;

/// One authored track of a layout axis, in logical points.
///
/// This is the authored spelling only. The Mosaic owner the layout lowers
/// into judges what is admissible, so a zero extent, a zero weight, or a
/// minimum above the maximum is written here and denied there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiLayoutTrack {
    Fixed {
        extent: u16,
    },
    Flexible {
        weight: u16,
        min: u16,
        max: Option<u16>,
    },
}

/// The column and row a member's cell starts at, and how many tracks it
/// spans on each axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiLayoutCell {
    column: u16,
    row: u16,
    column_span: u16,
    row_span: u16,
}

impl UiLayoutCell {
    pub const fn at(column: u16, row: u16) -> Self {
        Self::spanning(column, row, 1, 1)
    }

    pub const fn spanning(column: u16, row: u16, column_span: u16, row_span: u16) -> Self {
        Self {
            column,
            row,
            column_span,
            row_span,
        }
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

/// One authored grid: column and row tracks, the gaps between them, the
/// padding around them, and the cell each member component takes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiLayoutGrid {
    columns: Box<[UiLayoutTrack]>,
    rows: Box<[UiLayoutTrack]>,
    column_gap: u16,
    row_gap: u16,
    inline_padding: u16,
    block_padding: u16,
    members: Vec<(UiDslComponentReference, UiLayoutCell)>,
}

impl UiLayoutGrid {
    pub fn new(
        columns: impl IntoIterator<Item = UiLayoutTrack>,
        rows: impl IntoIterator<Item = UiLayoutTrack>,
    ) -> Self {
        Self {
            columns: columns.into_iter().collect(),
            rows: rows.into_iter().collect(),
            column_gap: 0,
            row_gap: 0,
            inline_padding: 0,
            block_padding: 0,
            members: Vec::new(),
        }
    }

    pub fn with_gaps(mut self, column_gap: u16, row_gap: u16) -> Self {
        self.column_gap = column_gap;
        self.row_gap = row_gap;
        self
    }

    /// Keeps the tracks `inline` points from the container's left and right
    /// edges and `block` points from its top and bottom.
    pub fn with_padding(mut self, inline: u16, block: u16) -> Self {
        self.inline_padding = inline;
        self.block_padding = block;
        self
    }

    pub fn with_member(mut self, component: UiDslComponentReference, cell: UiLayoutCell) -> Self {
        self.members.push((component, cell));
        self
    }

    pub fn columns(&self) -> &[UiLayoutTrack] {
        &self.columns
    }

    pub fn rows(&self) -> &[UiLayoutTrack] {
        &self.rows
    }

    pub const fn column_gap(&self) -> u16 {
        self.column_gap
    }

    pub const fn row_gap(&self) -> u16 {
        self.row_gap
    }

    pub const fn inline_padding(&self) -> u16 {
        self.inline_padding
    }

    pub const fn block_padding(&self) -> u16 {
        self.block_padding
    }

    /// The members in the order they were authored. The order is spelling
    /// only: the layout the runtime lowers them into holds them as a set, so
    /// reordering members changes the package identity but not the layout.
    pub fn members(&self) -> impl Iterator<Item = (&UiDslComponentReference, UiLayoutCell)> {
        self.members
            .iter()
            .map(|(component, cell)| (component, *cell))
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>) {
        for tracks in [&self.columns, &self.rows] {
            super::declaration::count(bytes, tracks.len());
            for track in tracks.iter() {
                encode_track(bytes, *track);
            }
        }
        for value in [
            self.column_gap,
            self.row_gap,
            self.inline_padding,
            self.block_padding,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        super::declaration::count(bytes, self.members.len());
        for (component, cell) in &self.members {
            super::declaration::text(bytes, component.as_str());
            for value in [cell.column, cell.row, cell.column_span, cell.row_span] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
}

fn encode_track(bytes: &mut Vec<u8>, track: UiLayoutTrack) {
    match track {
        UiLayoutTrack::Fixed { extent } => {
            bytes.push(1);
            bytes.extend_from_slice(&extent.to_le_bytes());
        }
        UiLayoutTrack::Flexible { weight, min, max } => {
            bytes.push(2);
            bytes.extend_from_slice(&weight.to_le_bytes());
            bytes.extend_from_slice(&min.to_le_bytes());
            super::declaration::optional(bytes, max);
        }
    }
}
