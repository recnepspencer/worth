//! Exact physical subjects consulted by each selected adjacency reader.
use super::projection_adjacency_doors::Door;
#[derive(Clone, Copy, Debug)]
pub(super) enum Subject {
    AnchorHead,
    AnchorHistory,
    RemoteHead,
    RemoteHistory,
    AnchorSymbols,
    RemoteSymbols,
}
impl Subject {
    pub(super) const ALL: [Self; 6] = [
        Self::AnchorHead,
        Self::AnchorHistory,
        Self::RemoteHead,
        Self::RemoteHistory,
        Self::AnchorSymbols,
        Self::RemoteSymbols,
    ];
    pub(super) fn remote(self) -> bool {
        matches!(
            self,
            Self::RemoteHead | Self::RemoteHistory | Self::RemoteSymbols
        )
    }
    pub(super) fn symbols(self) -> bool {
        matches!(self, Self::AnchorSymbols | Self::RemoteSymbols)
    }
    pub(super) fn history(self) -> bool {
        matches!(self, Self::AnchorHistory | Self::RemoteHistory)
    }
    pub(super) fn required(self, door: Door, historical: bool) -> bool {
        (!self.remote() || door.incoming()) && (!self.history() || historical)
    }
}
