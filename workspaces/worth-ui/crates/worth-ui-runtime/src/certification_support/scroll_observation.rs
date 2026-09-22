#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollObservationCertificationOutcome {
    Applied {
        source: worth_ui_host_contract::UiHostScrollDeltaSource,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
        requested_inline_subpixels: i64,
        requested_block_subpixels: i64,
        owners_visited: u16,
    },
    Denied(UiScrollObservationCertificationDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollObservationCertificationDenial {
    Targeting,
    PresentedSurfaceFallbackIsAmbiguous,
    MountedBasisUnavailable,
    Ownership,
    NoDeclaredScrollOwner,
    AllocationUnavailable,
    ViewportUnavailable,
    BoundsOutOfRange,
    DeltaOutOfRange,
    Route,
    SettleUnpublished,
    AxisHeldByChromeDrag,
    PendingChromeRelease,
    /// A latched gesture outlived the frame's willingness to admit its owner,
    /// which is what an accepted modal Portal does to the content behind it.
    LatchedOwnerNotAdmitted,
    /// Mounted geometry refused the displayed pose, so the route was not
    /// committed either.
    Geometry,
    PendingGeometryPublication,
    /// A wheel reported in lines latched to an owner that declares no line
    /// extent, so the notch had no distance to travel.
    OwnerDeclaresNoLineExtent,
}

pub trait WorthUiScrollObservationCertificationExt {
    fn scroll_observations_for_certification(
        &self,
    ) -> Box<[UiScrollObservationCertificationOutcome]>;
}

impl WorthUiScrollObservationCertificationExt
    for crate::runtime::interaction::UiInteractionBatchReceipt
{
    fn scroll_observations_for_certification(
        &self,
    ) -> Box<[UiScrollObservationCertificationOutcome]> {
        self.scroll_observations()
            .iter()
            .map(|outcome| match outcome {
                crate::runtime::scroll::UiHostScrollObservationOutcome::Applied(receipt) => {
                    let crate::runtime::scroll::UiScrollDeltaCause::Host {
                        source,
                        phase,
                        precision,
                    } = receipt.cause()
                    else {
                        unreachable!("host observation retains host scroll cause")
                    };
                    let (inline, block) = receipt.transitions().iter().fold(
                        (0_i64, 0_i64),
                        |(inline, block), transition| {
                            let delta = transition.consumed();
                            (
                                inline + delta.inline_subpixels(),
                                block + delta.block_subpixels(),
                            )
                        },
                    );
                    let remainder = receipt.remainder();
                    UiScrollObservationCertificationOutcome::Applied {
                        source,
                        phase,
                        precision,
                        requested_inline_subpixels: inline + remainder.inline_subpixels(),
                        requested_block_subpixels: block + remainder.block_subpixels(),
                        owners_visited: receipt.owners_visited(),
                    }
                }
                crate::runtime::scroll::UiHostScrollObservationOutcome::Denied(denial) => {
                    UiScrollObservationCertificationOutcome::Denied(map_denial(*denial))
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

fn map_denial(
    denial: crate::runtime::scroll::UiHostScrollObservationDenial,
) -> UiScrollObservationCertificationDenial {
    use crate::runtime::scroll::UiHostScrollObservationDenial as Denial;
    match denial {
        Denial::Targeting(_) => UiScrollObservationCertificationDenial::Targeting,
        Denial::PresentedSurfaceFallbackIsAmbiguous => {
            UiScrollObservationCertificationDenial::PresentedSurfaceFallbackIsAmbiguous
        }
        Denial::MountedBasisUnavailable => {
            UiScrollObservationCertificationDenial::MountedBasisUnavailable
        }
        Denial::Ownership(_) => UiScrollObservationCertificationDenial::Ownership,
        Denial::NoDeclaredScrollOwner => {
            UiScrollObservationCertificationDenial::NoDeclaredScrollOwner
        }
        Denial::AllocationUnavailable => {
            UiScrollObservationCertificationDenial::AllocationUnavailable
        }
        Denial::ViewportUnavailable => UiScrollObservationCertificationDenial::ViewportUnavailable,
        Denial::BoundsOutOfRange => UiScrollObservationCertificationDenial::BoundsOutOfRange,
        Denial::DeltaOutOfRange => UiScrollObservationCertificationDenial::DeltaOutOfRange,
        Denial::Route(_) => UiScrollObservationCertificationDenial::Route,
        Denial::SettleUnpublished => UiScrollObservationCertificationDenial::SettleUnpublished,
        Denial::AxisHeldByChromeDrag => {
            UiScrollObservationCertificationDenial::AxisHeldByChromeDrag
        }
        Denial::PendingChromeRelease => {
            UiScrollObservationCertificationDenial::PendingChromeRelease
        }
        Denial::LatchedOwnerNotAdmitted(_) => {
            UiScrollObservationCertificationDenial::LatchedOwnerNotAdmitted
        }
        Denial::Geometry(_) => UiScrollObservationCertificationDenial::Geometry,
        Denial::PendingGeometryPublication => {
            UiScrollObservationCertificationDenial::PendingGeometryPublication
        }
        Denial::OwnerDeclaresNoLineExtent => {
            UiScrollObservationCertificationDenial::OwnerDeclaresNoLineExtent
        }
    }
}
