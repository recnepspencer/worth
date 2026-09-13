use sha2::{Digest, Sha256};
use worth_ui_host_native::{
    UiNativeClientPresentationSemanticChange as SemanticChange,
    UiNativeClientPresentationSemanticFrontierObservation as Frontier,
};

use super::{require, Phase5LocalityAxis, Phase5LocalityCase};

pub(super) fn require_exact(
    case: Phase5LocalityCase,
    frontier: &Frontier,
    authored_mounted_instances: &[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation],
) -> Result<(), String> {
    require_identity_set(case, frontier, authored_mounted_instances)?;
    let expected_content = match case.axis() {
        Phase5LocalityAxis::Content => "A\u{00a0}A",
        Phase5LocalityAxis::AtlasMiss | Phase5LocalityAxis::UploadCompletion => "AAAB",
        Phase5LocalityAxis::PinRelease => "AB",
        _ => "AAAA",
    };
    let target_foreground: &[u8] = match case.axis() {
        Phase5LocalityAxis::PaintBoundary => &[229, 57, 53, 255, 240, 242, 245, 255],
        _ => &[229, 57, 53, 255],
    };
    let expected_content = digest_bytes(expected_content.as_bytes());
    let target_foreground = digest_bytes(target_foreground);
    let base_foreground = digest_bytes(&[240, 242, 245, 255]);
    let subscribers = frontier.subscribers();
    if let Some(subscriber) = subscribers.iter().find(|subscriber| {
        subscriber.layout_digest() == [0; 32] || subscriber.foreground_digest() == [0; 32]
    }) {
        return Err(format!(
            "semantic subscriber lost exact mechanic evidence: content={:02x?} layout={:02x?} foreground={:02x?}",
            subscriber.content_digest(),
            subscriber.layout_digest(),
            subscriber.foreground_digest(),
        ));
    }
    let authored_content = subscribers
        .iter()
        .filter(|subscriber| subscriber.content_digest() == expected_content)
        .count();
    let fallback_content = digest_bytes(b"active-application-presentation");
    let removal_fallback = subscribers
        .iter()
        .filter(|subscriber| subscriber.content_digest() == fallback_content)
        .count();
    let expected_authored = if matches!(
        case.axis(),
        Phase5LocalityAxis::Dpi | Phase5LocalityAxis::UploadCompletion
    ) {
        case.retained_paragraphs()
    } else {
        1
    };
    let expected_fallback = subscribers.len().saturating_sub(expected_authored);
    require(
        authored_content == expected_authored && removal_fallback == expected_fallback,
        "semantic frontier selected the wrong authored content set",
    )?;
    require_exact_foregrounds(case, frontier, target_foreground, base_foreground)
}

fn require_identity_set(
    case: Phase5LocalityCase,
    frontier: &Frontier,
    authored_mounted_instances: &[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation],
) -> Result<(), String> {
    let identities = frontier
        .subscribers()
        .iter()
        .map(|subscriber| {
            (
                subscriber.mounted_instance().unwrap_or_default(),
                subscriber.semantic_slot().unwrap_or_default(),
                subscriber.collection_row(),
            )
        })
        .collect::<std::collections::HashSet<_>>();
    let indexes: Box<dyn Iterator<Item = usize>> = match case.axis() {
        Phase5LocalityAxis::Dpi | Phase5LocalityAxis::UploadCompletion => {
            Box::new(0..case.retained_paragraphs())
        }
        _ => Box::new(std::iter::once(case.target_index())),
    };
    let expected = indexes
        .map(|index| {
            let authored = digest_bytes(case.authored_identity(index).as_bytes());
            authored_mounted_instances
                .iter()
                .find(|observation| observation.authored_semantic_identity_digest() == authored)
                .map(|observation| (index, observation.mounted_instance()))
                .ok_or_else(|| {
                    format!("application owner omitted the mounted admission for paragraph {index}")
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|(_, instance)| {
            let slots: &[u16] = if case.axis() == Phase5LocalityAxis::PaintBoundary {
                &[0]
            } else {
                &[0, u16::MAX]
            };
            slots
                .iter()
                .copied()
                .map(move |slot| (instance, slot, None))
        })
        .collect::<std::collections::HashSet<_>>();
    if identities == expected {
        Ok(())
    } else {
        Err(format!(
            "Query subscriber identity set is not the exact separately selected mechanic set: observed={identities:?} expected={expected:?}"
        ))
    }
}

fn require_exact_foregrounds(
    case: Phase5LocalityCase,
    frontier: &Frontier,
    target_foreground: [u8; 32],
    base_foreground: [u8; 32],
) -> Result<(), String> {
    let target = frontier
        .subscribers()
        .iter()
        .filter(|subscriber| subscriber.foreground_digest() == target_foreground)
        .count();
    let base = frontier
        .subscribers()
        .iter()
        .filter(|subscriber| subscriber.foreground_digest() == base_foreground)
        .count();
    let all_mechanics = matches!(
        (case.axis(), frontier.change()),
        (Phase5LocalityAxis::Dpi, SemanticChange::Dpi)
            | (
                Phase5LocalityAxis::UploadCompletion,
                SemanticChange::Content
            )
            | (
                Phase5LocalityAxis::UploadCompletion,
                SemanticChange::UploadCompletion
            )
    );
    let expected_target = if case.axis() == Phase5LocalityAxis::PaintBoundary {
        1
    } else {
        2
    };
    let expected_base = if all_mechanics {
        case.retained_paragraphs().saturating_sub(1) * 2
    } else {
        0
    };
    if target == expected_target && base == expected_base {
        Ok(())
    } else {
        Err(format!(
            "semantic frontier selected the wrong authored target or fanout set: target={target}/{expected_target} base={base}/{expected_base} digests={:?}",
            frontier
                .subscribers()
                .iter()
                .map(|subscriber| subscriber.foreground_digest())
                .collect::<Vec<_>>()
        ))
    }
}

fn digest_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
