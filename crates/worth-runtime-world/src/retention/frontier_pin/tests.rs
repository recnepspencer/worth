use super::*;
use worth_foundational::facade::HomeIncarnation;

fn sealed() -> HomeRootManifest {
    let at = |position| PhysicalRootPointer::from_retention_owner(7, position);
    HomeRootManifest::seal(
        HomeIncarnation::initial(),
        7,
        HomeFrontierComponents {
            native_root: at(3),
            world_root: at(5),
            signal_root: at(2),
            product_map_root: at(4),
        },
        [9; 32],
    )
}

#[test]
fn sealed_manifest_round_trips_and_binds_the_product_map_root() {
    let manifest = sealed();
    let decoded = HomeRootManifest::decode(&manifest.encode()).expect("sealed manifest decodes");
    assert_eq!(decoded, manifest);
    assert_eq!(decoded.product_map_root().position(), 4);
    assert_eq!(decoded.frontier_sequence(), 7);
}

#[test]
fn any_tampered_byte_breaks_the_seal() {
    let bytes = sealed().encode();
    for index in [10, 66, 74, 90, 130] {
        let mut tampered = bytes.clone();
        tampered[index] ^= 1;
        assert_eq!(HomeRootManifest::decode(&tampered), None, "byte {index}");
    }
    assert_eq!(HomeRootManifest::decode(&bytes[1..]), None);
}
