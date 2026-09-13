//! Independently retained identity goldens for the authored locality fixture.

use worth_ui_host_native::UiNativeClientPresentationSemanticSubscriberObservation as Subscriber;

use super::{Phase5LocalityAxis as Axis, Phase5LocalityCase};

pub(super) fn require_exact(
    case: Phase5LocalityCase,
    subscriber: Subscriber,
) -> Result<(), String> {
    let slot = subscriber
        .semantic_slot()
        .ok_or_else(|| "matrix subscriber omitted its semantic slot".to_owned())?;
    let (layout, raster) = expected(case.axis(), slot)
        .ok_or_else(|| format!("matrix subscriber used an unexpected semantic slot {slot}"))?;
    let observed_layout = hex(subscriber.layout_digest());
    let observed_raster = hex(subscriber.raster_key_set_digest());
    if observed_layout == layout && observed_raster == raster {
        Ok(())
    } else {
        Err(format!(
            "matrix subscriber layout/raster identity differs from the authored fixture model: slot={slot} layout={observed_layout}/{layout} raster={observed_raster}/{raster}"
        ))
    }
}

fn expected(axis: Axis, slot: u16) -> Option<(&'static str, &'static str)> {
    const POSTURE: (&str, &str) = (
        "5dc6367baf4f0aa633e84a09969381642aa694da92a7a2f6da6cc5f2bf2d2234",
        "fd689571de4cf11b9c28d194522ecc58cb59cfb9588d30e5ffbe4032fc662d7a",
    );
    const DPI_POSTURE: (&str, &str) = (
        "5dc6367baf4f0aa633e84a09969381642aa694da92a7a2f6da6cc5f2bf2d2234",
        "5afbc362792a988dc3b22358e665340c81a557d9d6e610da3caf45566aa82d02",
    );
    const WIDTH_POSTURE: (&str, &str) = (
        "b60c3df578a1e1ce687097e064462e5f340b7516fbc7964fc5043544b6b43c76",
        "70f177204ca57c5f9b37ac2a418b2d8bd32b409a867efe515102b17c2b072f36",
    );
    if slot == u16::MAX {
        return Some(match axis {
            Axis::Dpi => DPI_POSTURE,
            Axis::Width => WIDTH_POSTURE,
            _ => POSTURE,
        });
    }
    if slot != 0 {
        return None;
    }
    Some(match axis {
        Axis::Content => (
            "3084ccb3f74faf91308d76d829a136f3be06a7bf5b13d849be4d3a8a90e423f1",
            "d266f094569e855b3aa1f40714716652285d25c2bf13a8be8a557232b69bb6dd",
        ),
        Axis::Width => (
            "f6ef4a624de34f8b2bab01ddb5c977d7bc36515dc103addcd83ed891c4bc9ade",
            "d5bbca4eafc3ca958b0d966006e90c39fc7e994e4d2b66c010aa0a13becc68b4",
        ),
        Axis::PaintBoundary => (
            "7f5155e5f19df1b6d31a28ea45f63726bd55bf54cc086b7895aad0829bbe55c9",
            "d5bbca4eafc3ca958b0d966006e90c39fc7e994e4d2b66c010aa0a13becc68b4",
        ),
        Axis::Dpi => (
            "c4fef77c4bbbfbbac670f39c90766a24c647b1422a97b0cc7d131c73b846a6aa",
            "732d677d596e47759b008e8e425118ea7799c183ebffb202056588f0e7e6795c",
        ),
        Axis::AtlasMiss | Axis::UploadCompletion => (
            "ce7fdb91cf3e2eb636c6c40c4c36afdc920e570836fdda33c8c17494625dfb9e",
            "1b18917bc89ce63391d439b4a188cd88988343f35f5dfe0333a42c9fb7453cad",
        ),
        Axis::PinRelease => (
            "c5248825198111ddff8ba780e4e099229ed92fbc5328bc2768073e58c29b93ad",
            "18fad3abe888a275a8cd6e0d63b04f3b278c61c06eaa56f3f8349ceba4ce0992",
        ),
    })
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
