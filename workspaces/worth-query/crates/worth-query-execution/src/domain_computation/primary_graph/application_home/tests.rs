use super::{
    ApplicationHome, WorthQueryHomeAbsent, WorthQueryHomeForm, WorthQueryReopenDeferral,
    WorthQueryReturnPoint, WorthQueryStateOwner,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint;

fn image() -> WorthQueryApplicationCheckpoint {
    WorthQueryApplicationCheckpoint::from_untrusted_bytes(vec![7_u8; 16])
}

#[test]
fn a_memory_home_starts_empty() {
    assert_eq!(ApplicationHome::memory().closed_image(), Ok(None));
}

#[test]
fn an_image_settles_into_a_memory_home() {
    let home = ApplicationHome::holding(image());
    assert_eq!(home.closed_image(), Ok(Some(&image())));
}

#[test]
fn a_path_home_answers_with_its_deferral_and_touches_no_filesystem() {
    let path = std::env::temp_dir().join("worth-query-deferred-application-home");
    let home = ApplicationHome::at(&path);
    assert_eq!(
        home.closed_image(),
        Err(WorthQueryHomeAbsent {
            form: WorthQueryHomeForm::At,
            deferral: WorthQueryReopenDeferral {
                owner: WorthQueryStateOwner::Relational,
                return_point: WorthQueryReturnPoint::RelationalOnStore,
            },
        })
    );
    assert!(!path.exists(), "naming a home must create nothing");
}

#[test]
fn a_home_reports_whether_it_holds_an_image_without_exposing_it() {
    assert_eq!(
        format!("{:?}", ApplicationHome::memory()),
        "ApplicationHome::Memory { holds_image: false }"
    );
    assert_eq!(
        format!("{:?}", ApplicationHome::holding(image())),
        "ApplicationHome::Memory { holds_image: true }"
    );
}

#[cfg(feature = "test-durability-faults")]
#[test]
fn the_durability_seam_carries_image_bytes_and_never_reads_a_path_home() {
    let bytes = image().bytes().to_vec();
    let home = ApplicationHome::memory_from_image_bytes_for_durability_test(bytes.clone());
    assert_eq!(home.image_bytes_for_durability_test(), Some(&bytes[..]));
    assert_eq!(
        ApplicationHome::memory().image_bytes_for_durability_test(),
        None
    );
    let path_home = ApplicationHome::at("deferred-application-home");
    assert_eq!(path_home.image_bytes_for_durability_test(), None);
    assert!(path_home
        .rewrite_authenticated_body_for_durability_test(|_| {})
        .is_none());
}
