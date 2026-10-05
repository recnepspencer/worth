use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationBasisReleaseOutcome, WorthQueryApplicationQueryAccessReceipt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedApplicationQueryReleasePosture {
    Released,
    SharedCustodyRetained,
    ReleaseFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedApplicationQueryResultBufferRelease {
    Missing,
    Released {
        limit_bytes: usize,
        peak_bytes: usize,
    },
    ReleaseFailed {
        limit_bytes: usize,
        peak_bytes: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedApplicationQueryTerminalRelease {
    application_basis: WorthQueryPublishedApplicationQueryReleasePosture,
    graph_read_basis: WorthQueryPublishedApplicationQueryReleasePosture,
    result_buffer: WorthQueryPublishedApplicationQueryResultBufferRelease,
    released_graph_capacity_reservation_count: usize,
}

impl WorthQueryPublishedApplicationQueryTerminalRelease {
    pub(super) fn capture(terminal: &WorthQueryApplicationQueryAccessReceipt) -> Self {
        let completion = terminal.read_completion();
        Self {
            application_basis: release_posture(terminal.basis_released()),
            graph_read_basis: match completion.basis_release().outcome() {
                WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained => {
                    WorthQueryPublishedApplicationQueryReleasePosture::SharedCustodyRetained
                }
                outcome => release_posture(outcome.released()),
            },
            result_buffer: match terminal.result_buffer() {
                None => WorthQueryPublishedApplicationQueryResultBufferRelease::Missing,
                Some(buffer) if buffer.released() => {
                    WorthQueryPublishedApplicationQueryResultBufferRelease::Released {
                        limit_bytes: buffer.limit_bytes(),
                        peak_bytes: buffer.peak_bytes(),
                    }
                }
                Some(buffer) => {
                    WorthQueryPublishedApplicationQueryResultBufferRelease::ReleaseFailed {
                        limit_bytes: buffer.limit_bytes(),
                        peak_bytes: buffer.peak_bytes(),
                    }
                }
            },
            released_graph_capacity_reservation_count: completion
                .release()
                .released_reservation_count(),
        }
    }

    pub const fn application_basis(self) -> WorthQueryPublishedApplicationQueryReleasePosture {
        self.application_basis
    }

    pub const fn graph_read_basis(self) -> WorthQueryPublishedApplicationQueryReleasePosture {
        self.graph_read_basis
    }

    pub const fn result_buffer(self) -> WorthQueryPublishedApplicationQueryResultBufferRelease {
        self.result_buffer
    }

    pub const fn released_graph_capacity_reservation_count(self) -> usize {
        self.released_graph_capacity_reservation_count
    }

    pub const fn resources_released(self) -> bool {
        matches!(
            self.application_basis,
            WorthQueryPublishedApplicationQueryReleasePosture::Released
        ) && matches!(
            self.graph_read_basis,
            WorthQueryPublishedApplicationQueryReleasePosture::Released
        ) && matches!(
            self.result_buffer,
            WorthQueryPublishedApplicationQueryResultBufferRelease::Released { .. }
        ) && self.released_graph_capacity_reservation_count == 1
    }
}

const fn release_posture(released: bool) -> WorthQueryPublishedApplicationQueryReleasePosture {
    if released {
        WorthQueryPublishedApplicationQueryReleasePosture::Released
    } else {
        WorthQueryPublishedApplicationQueryReleasePosture::ReleaseFailed
    }
}

#[cfg(test)]
mod aggregate_tests;
