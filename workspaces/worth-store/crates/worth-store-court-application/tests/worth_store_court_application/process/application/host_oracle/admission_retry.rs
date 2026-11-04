//! Retry the same D8 predecessor after a genuine nonempty admitted prefix.
use super::*;
use super::{
    protocol::{Disposition, ReleasedOriginalProjection, Request},
    session::Session,
};
use std::path::Path;
mod readmission;
const KEYS: [u64; 4] = [0x8810, 0x8811, 0x8812, 0x8813];

#[test]
fn partial_released_key_admission_retries_same_home_without_duplicate_retention() {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    let (originals, scale) = originals(&location, root.path());
    readmission::retry(&location, &originals, scale);
}
fn originals(location: &Path, root: &Path) -> (Vec<ReleasedOriginalProjection>, u64) {
    let mut originals = Vec::new();
    let mut scale = 1;
    for (index, key) in KEYS.into_iter().enumerate() {
        let (mut session, _) = Session::start(location, &root.join(format!("original-{index}")), 0);
        let actual = session.send(Request::AbandonOriginal { key, value: key });
        let state = actual.state.unwrap();
        assert_eq!(state.custody.len(), 1);
        assert!(state.custody[0].original_effect.is_some());
        session.kill();
        let (mut session, _) = Session::start(location, &root.join(format!("release-{index}")), 0);
        let actual = session.send(Request::ReleaseOriginal {
            key,
            value: key,
            expect_continued: false,
        });
        assert!(actual.state.unwrap().custody.is_empty());
        assert_eq!(actual.contacts, 0);
        let actual = session.send(Request::ObserveReleased { key, value: key });
        let Some(Disposition::ReleasedOriginal(original)) = actual.disposition else {
            panic!("actual released source");
        };
        assert!(originals
            .iter()
            .all(|prior: &ReleasedOriginalProjection| prior.identity != original.identity));
        let quote = session.send(Request::ObserveReleasedCopyScale { key, value: key });
        let Some(Disposition::OriginalBindingCopyScale(bytes)) = quote.disposition else {
            panic!("actual original row-copy quote");
        };
        scale = scale.max(bytes);
        originals.push(original);
        session.kill();
    }
    (originals, scale)
}
