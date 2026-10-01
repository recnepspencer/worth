use crate::catalog::ArtifactFamilyAccessLane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrategyCapability {
    point: bool,
    range: bool,
    prefix: bool,
    scan: bool,
    streaming: bool,
    hot_path: bool,
    maintenance: bool,
    verifier: bool,
    terminal: bool,
}

impl StrategyCapability {
    // The fixed capability matrix is declared once and read as named columns at each call site.
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        point: bool,
        range: bool,
        prefix: bool,
        scan: bool,
        streaming: bool,
        hot_path: bool,
        maintenance: bool,
        verifier: bool,
        terminal: bool,
    ) -> Self {
        Self {
            point,
            range,
            prefix,
            scan,
            streaming,
            hot_path,
            maintenance,
            verifier,
            terminal,
        }
    }

    pub(crate) const fn btree_range() -> Self {
        Self::new(true, true, true, false, false, true, true, false, false)
    }

    pub(crate) const fn baseline_lsm_write_optimized() -> Self {
        Self::new(true, false, false, false, false, true, true, false, false)
    }

    pub(crate) const fn supports_point(self) -> bool {
        self.point
    }

    pub(crate) const fn supports_range(self) -> bool {
        self.range
    }

    pub(crate) const fn supports_prefix(self) -> bool {
        self.prefix
    }

    pub(crate) const fn supports_scan(self) -> bool {
        self.scan
    }

    pub(crate) const fn supports_streaming(self) -> bool {
        self.streaming
    }

    pub(crate) const fn allows_lane(self, lane: ArtifactFamilyAccessLane) -> bool {
        match lane {
            ArtifactFamilyAccessLane::HotPath => self.hot_path,
            ArtifactFamilyAccessLane::MaintenancePath => self.maintenance,
            ArtifactFamilyAccessLane::VerifierPath => self.verifier,
            ArtifactFamilyAccessLane::TerminalPath => self.terminal,
        }
    }
}
