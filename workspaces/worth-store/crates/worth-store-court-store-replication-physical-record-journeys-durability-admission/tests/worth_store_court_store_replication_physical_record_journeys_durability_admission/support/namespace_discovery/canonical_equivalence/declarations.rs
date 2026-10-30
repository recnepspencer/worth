pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::MediaOwnedPhysicalRuntime;
pub(crate) use {
    worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_comparison,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalMismatchKind,
    worth_foundational::CanonicalizationRuleVersion,
};
pub(crate) use {
    worth_store_physical_format::store_namespace::prepare_external_store_namespace_identity_canonical_basis,
    worth_store_physical_format::store_namespace::prepare_store_namespace_identity_canonical_basis,
    worth_store_physical_format::store_namespace::ExternalStoreNamespaceIdentityMeaning,
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityCanonicalMeaning,
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityPublicationPosture,
};

pub(crate) fn canonical_mismatch_loci(media: &MediaOwnedPhysicalRuntime) -> String {
    let namespace_version = required_env_u16("WORTH_STORE_C4_OBSERVED_NAMESPACE_VERSION");
    let encoding_version = required_env_u16("WORTH_STORE_C4_OBSERVED_ENCODING_VERSION");
    let external_identity = decode_hex_16(
        &std::env::var("WORTH_STORE_C4_OBSERVED_IDENTITY").expect("observer identity"),
    );
    let product =
        worth_store_physical_format::store_namespace::StoreNamespaceIdentityCanonicalMeaning::from_published_identity(media.store_identity());
    let external = worth_store_physical_format::store_namespace::ExternalStoreNamespaceIdentityMeaning::observed_published(
        namespace_version,
        encoding_version,
        external_identity,
    );
    assert!(matches!(
        compare(product, external),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
    let mut changed_identity = external_identity;
    changed_identity[0] ^= 1;
    let mutants = [
        (
            worth_store_physical_format::store_namespace::ExternalStoreNamespaceIdentityMeaning::observed_published(
                namespace_version + 1,
                encoding_version,
                external_identity,
            ),
            "namespace.version",
        ),
        (
            worth_store_physical_format::store_namespace::ExternalStoreNamespaceIdentityMeaning::observed_published(
                namespace_version,
                encoding_version + 1,
                external_identity,
            ),
            "encoding.version",
        ),
        (
            worth_store_physical_format::store_namespace::ExternalStoreNamespaceIdentityMeaning::observed_published(
                namespace_version,
                encoding_version,
                changed_identity,
            ),
            "identity",
        ),
        (
            external.with_publication_posture(
                worth_store_physical_format::store_namespace::StoreNamespaceIdentityPublicationPosture::StagedCandidate,
            ),
            "publication.posture",
        ),
    ];
    mutants
        .into_iter()
        .map(|(mutant, locus)| assert_canonical_mismatch(product, mutant, locus))
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn assert_canonical_mismatch(
    product: StoreNamespaceIdentityCanonicalMeaning,
    mutant: ExternalStoreNamespaceIdentityMeaning,
    expected_locus: &'static str,
) -> &'static str {
    match compare(product, mutant) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::ValueMismatch
            );
            assert_eq!(
                mismatch.left_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    expected_locus.into()
                ))
            );
            expected_locus
        }
        other => panic!("canonical-locus mutant escaped at {expected_locus}: {other:?}"),
    }
}

pub(crate) fn compare(
    product: StoreNamespaceIdentityCanonicalMeaning,
    external: ExternalStoreNamespaceIdentityMeaning,
) -> CanonicalComparisonOutcome {
    let version =
        worth_foundational::CanonicalizationRuleVersion::new("store.namespace.identity.v1")
            .unwrap();
    let left = match prepare_store_namespace_identity_canonical_basis(version.clone(), product) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("product canonical basis failed: {other:?}"),
    };
    let right = match prepare_external_store_namespace_identity_canonical_basis(version, external) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("external canonical basis failed: {other:?}"),
    };
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("canonical comparison preparation failed"),
    };
    compare_canonical_basis(&ready)
}

pub(crate) fn required_env_u16(name: &str) -> u16 {
    std::env::var(name).unwrap().parse().unwrap()
}

pub(crate) fn decode_hex_16(text: &str) -> [u8; 16] {
    assert_eq!(text.len(), 32);
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
