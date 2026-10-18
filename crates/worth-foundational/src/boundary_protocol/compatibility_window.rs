use super::{
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolUnsupportedVersionPosture,
    BoundaryProtocolVersion,
};

/// One external consumer's independently declared accepted version interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundaryProtocolCompatibilityWindow {
    earliest: BoundaryProtocolVersion,
    latest: BoundaryProtocolVersion,
    retired_before: Option<BoundaryProtocolVersion>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryProtocolCompatibilityWindowDenial {
    Reversed,
}

impl BoundaryProtocolCompatibilityWindow {
    /// Declares an inclusive compatibility interval.
    ///
    /// # Panics
    ///
    /// Panics when `earliest` follows `latest`.
    pub const fn inclusive(
        earliest: BoundaryProtocolVersion,
        latest: BoundaryProtocolVersion,
    ) -> Self {
        if earliest.get() > latest.get() {
            panic!("boundary protocol compatibility window is reversed");
        }
        Self {
            earliest,
            latest,
            retired_before: None,
        }
    }

    /// Builds a compatibility interval from runtime configuration without a
    /// panic edge.
    pub const fn try_inclusive(
        earliest: BoundaryProtocolVersion,
        latest: BoundaryProtocolVersion,
    ) -> Result<Self, BoundaryProtocolCompatibilityWindowDenial> {
        if earliest.get() > latest.get() {
            return Err(BoundaryProtocolCompatibilityWindowDenial::Reversed);
        }
        Ok(Self {
            earliest,
            latest,
            retired_before: None,
        })
    }

    /// Retires versions strictly below `version` while preserving the declared
    /// interval for diagnostics and deliberate future-window evolution.
    pub const fn retire_before(mut self, version: BoundaryProtocolVersion) -> Self {
        self.retired_before = Some(version);
        self
    }

    pub const fn admit(
        self,
        produced: BoundaryProtocolVersion,
    ) -> Result<BoundaryProtocolVersion, BoundaryProtocolUnsupportedVersion> {
        if produced.get() < self.earliest.get() {
            return Err(BoundaryProtocolUnsupportedVersion::new(
                produced,
                BoundaryProtocolUnsupportedVersionPosture::PredatesWindow,
            ));
        }
        if produced.get() > self.latest.get() {
            return Err(BoundaryProtocolUnsupportedVersion::new(
                produced,
                BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow,
            ));
        }
        if let Some(retired_before) = self.retired_before
            && produced.get() < retired_before.get()
        {
            return Err(BoundaryProtocolUnsupportedVersion::new(
                produced,
                BoundaryProtocolUnsupportedVersionPosture::Retired,
            ));
        }
        Ok(produced)
    }

    pub const fn earliest(self) -> BoundaryProtocolVersion {
        self.earliest
    }

    pub const fn latest(self) -> BoundaryProtocolVersion {
        self.latest
    }

    pub const fn retired_before(self) -> Option<BoundaryProtocolVersion> {
        self.retired_before
    }
}

#[cfg(test)]
mod tests {
    use super::BoundaryProtocolCompatibilityWindow;
    use crate::boundary_protocol::{
        BoundaryProtocolUnsupportedVersionPosture, BoundaryProtocolVersion,
    };

    #[test]
    fn coexistence_downgrade_future_and_retirement_are_distinct() {
        let window = BoundaryProtocolCompatibilityWindow::inclusive(
            BoundaryProtocolVersion::new(1),
            BoundaryProtocolVersion::new(2),
        );
        assert_eq!(
            window.admit(BoundaryProtocolVersion::new(1)),
            Ok(BoundaryProtocolVersion::new(1))
        );
        assert_eq!(
            window.admit(BoundaryProtocolVersion::new(2)),
            Ok(BoundaryProtocolVersion::new(2))
        );
        let downgrade = BoundaryProtocolCompatibilityWindow::inclusive(
            BoundaryProtocolVersion::new(2),
            BoundaryProtocolVersion::new(3),
        )
        .admit(BoundaryProtocolVersion::new(1))
        .unwrap_err();
        assert_eq!(
            downgrade.posture(),
            BoundaryProtocolUnsupportedVersionPosture::PredatesWindow
        );
        let future = window.admit(BoundaryProtocolVersion::new(3)).unwrap_err();
        assert_eq!(
            future.posture(),
            BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow
        );

        let retired = window
            .retire_before(BoundaryProtocolVersion::new(2))
            .admit(BoundaryProtocolVersion::new(1))
            .unwrap_err();
        assert_eq!(
            retired.posture(),
            BoundaryProtocolUnsupportedVersionPosture::Retired
        );
        assert_eq!(
            window
                .retire_before(BoundaryProtocolVersion::new(2))
                .admit(BoundaryProtocolVersion::new(2)),
            Ok(BoundaryProtocolVersion::new(2))
        );
    }

    #[test]
    fn runtime_window_construction_denies_reversed_configuration() {
        assert_eq!(
            BoundaryProtocolCompatibilityWindow::try_inclusive(
                BoundaryProtocolVersion::new(2),
                BoundaryProtocolVersion::new(1),
            ),
            Err(super::BoundaryProtocolCompatibilityWindowDenial::Reversed)
        );
    }
}
