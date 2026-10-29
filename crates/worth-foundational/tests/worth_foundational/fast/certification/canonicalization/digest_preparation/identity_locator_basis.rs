use worth_foundational::{
    diagnostic_mask_locator_canonical_basis_entries, identity_canonical_basis_entries,
    locator_canonical_basis_entries, mutation_mask_locator_canonical_basis_entries,
    prepare_canonical_basis_sequence, prepare_identity_for_canonical_basis,
    prepare_locator_for_canonical_basis, projection_mask_locator_canonical_basis_entries,
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, AspectValueLocator,
    BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator, BoundaryEpoch,
    BoundaryHandle, BoundaryMismatchLocator, BoundarySourceLocator, CanonicalBasisDomain,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalFieldPath,
    CanonicalIdentityInput, CanonicalIntegerWidth, CanonicalLocatorInput,
    CanonicalizationRuleVersion, DiagnosticMask, EquivalenceBasisId, FieldKey, LocatorAuthority,
    MutationMask, ProjectionMask,
};
use worth_proof::TransitionOutcome;

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("m2.identity-locator").expect("valid test version")
}

fn key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid aspect key")
}

fn field(value: &str) -> FieldKey {
    FieldKey::new(value).expect("valid field")
}

fn path(value: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::single(field(value))
}

#[test]
fn identity_basis_keeps_equal_storage_categories_distinct() {
    let handle = match prepare_identity_for_canonical_basis(
        version(),
        CanonicalIdentityInput::BoundaryHandle(BoundaryHandle::new(9)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("handle identity basis should be ready"),
    };
    let epoch = match prepare_identity_for_canonical_basis(
        version(),
        CanonicalIdentityInput::BoundaryEpoch(BoundaryEpoch::new(9)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("epoch identity basis should be ready"),
    };
    let equivalence = match prepare_identity_for_canonical_basis(
        version(),
        CanonicalIdentityInput::EquivalenceBasis(EquivalenceBasisId::new(9)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("equivalence identity basis should be ready"),
    };

    assert_eq!(handle.payload().domain(), CanonicalBasisDomain::Identity);
    assert_eq!(
        handle.payload().entries()[0].value(),
        epoch.payload().entries()[0].value()
    );
    assert_ne!(
        handle.payload().entries()[0].locus(),
        epoch.payload().entries()[0].locus()
    );
    assert_ne!(
        handle.payload().entries()[0].locus(),
        equivalence.payload().entries()[0].locus()
    );
    assert_eq!(
        handle.basis().basis().value().as_str(),
        handle.payload().version().as_str()
    );
    assert_eq!(
        identity_canonical_basis_entries(&handle),
        handle.payload().entries()
    );
}

#[test]
fn locator_basis_canonicalizes_source_and_mismatch_categories() {
    let artifact =
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(11), BoundaryArtifactField::Basis);
    let source = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Source(BoundarySourceLocator::boundary_artifact(artifact)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("source locator basis should be ready"),
    };
    let mismatch = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Mismatch(BoundaryMismatchLocator::boundary_artifact(artifact)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("mismatch locator basis should be ready"),
    };

    assert_eq!(source.payload().domain(), CanonicalBasisDomain::Locator);
    assert_ne!(source.payload().entries(), mismatch.payload().entries());
    assert!(source.payload().entries().iter().any(|entry| entry.locus()
        == &CanonicalBasisLocus::Named("source.boundary_artifact.field".into())));
    assert!(mismatch
        .payload()
        .entries()
        .iter()
        .any(|entry| entry.locus()
            == &CanonicalBasisLocus::Named("mismatch.boundary_artifact.field".into())));
    assert_eq!(
        locator_canonical_basis_entries(&source),
        source.payload().entries()
    );
}

#[test]
fn aspect_field_locator_basis_uses_semantic_path_not_debug_shape() {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        key("task.summary"),
        CanonicalFieldPath::new([field("body"), field("plain")]).expect("nonempty path"),
    );
    let ready = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::AspectField(locator),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("field locator basis should be ready"),
    };

    assert!(ready.payload().entries().iter().any(|entry| {
        entry.kind() == CanonicalBasisEntryKind::Locator
            && entry.locus() == &CanonicalBasisLocus::Named("aspect_field.field_path".into())
            && entry.value() == &CanonicalBasisValue::ExactText("body.plain".to_string().into())
    }));
}

#[test]
fn mask_locator_basis_preserves_mode_and_ordered_paths() {
    let projection = AspectMask::<ProjectionMask>::new([path("zeta"), path("alpha")]);
    let mutation = AspectMask::<MutationMask>::new([path("zeta"), path("alpha")]);
    let diagnostic = AspectMask::<DiagnosticMask>::new([path("zeta"), path("alpha")]);

    let projection_entries = projection_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::projection(
            LocatorAuthority::Projected,
            key("task.summary"),
            &projection,
        ),
    );
    let mutation_entries = mutation_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::mutation(
            LocatorAuthority::Projected,
            key("task.summary"),
            &mutation,
        ),
    );
    let diagnostic_entries = diagnostic_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::diagnostic(
            LocatorAuthority::Projected,
            key("task.summary"),
            &diagnostic,
        ),
    );

    assert_ne!(projection_entries, mutation_entries);
    assert_ne!(mutation_entries, diagnostic_entries);

    let ready = match prepare_canonical_basis_sequence(
        version(),
        CanonicalBasisDomain::Locator,
        projection_entries,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("mask locator entries should form a ready locator basis"),
    };

    assert_eq!(ready.payload().cost().entry_count(), 5);
    assert_eq!(
        ready.payload().entries()[3].value(),
        &CanonicalBasisValue::ExactText("alpha".to_string().into())
    );
    assert_eq!(
        ready.payload().entries()[4].value(),
        &CanonicalBasisValue::ExactText("zeta".to_string().into())
    );
}

#[test]
fn identity_and_locator_basis_do_not_collapse_adjacent_numeric_meanings() {
    let identity = match prepare_identity_for_canonical_basis(
        version(),
        CanonicalIdentityInput::BoundaryArtifact(BoundaryArtifactId::new(11)),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity basis should be ready"),
    };
    let locator = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::BoundaryArtifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(11),
            BoundaryArtifactField::Payload,
        )),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("locator basis should be ready"),
    };

    assert_eq!(
        identity.payload().entries()[0].value(),
        &CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 11,
        }
    );
    assert_eq!(identity.payload().domain(), CanonicalBasisDomain::Identity);
    assert_eq!(locator.payload().domain(), CanonicalBasisDomain::Locator);
    assert_ne!(
        identity.payload().entries()[0].locus(),
        locator.payload().entries()[0].locus()
    );
}

#[test]
fn aspect_locator_basis_preserves_authority_as_identity() {
    let authoritative = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Aspect(AspectLocator::new(
            LocatorAuthority::Authoritative,
            key("task.summary"),
        )),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("authoritative aspect locator basis should be ready"),
    };
    let projected = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Aspect(AspectLocator::new(
            LocatorAuthority::Projected,
            key("task.summary"),
        )),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("projected aspect locator basis should be ready"),
    };

    assert_ne!(
        authoritative.payload().entries(),
        projected.payload().entries()
    );
}

#[test]
fn value_locator_basis_is_distinct_from_plain_aspect_and_field_locator_basis() {
    let aspect = AspectLocator::new(LocatorAuthority::Authoritative, key("task.summary"));
    let field = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        key("task.summary"),
        CanonicalFieldPath::new([field("body"), field("plain")]).expect("non-empty path"),
    );

    let whole_value = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Value(AspectValueLocator::whole_aspect(aspect.clone())),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("whole-aspect value locator basis should be ready"),
    };
    let plain_aspect =
        match prepare_locator_for_canonical_basis(version(), CanonicalLocatorInput::Aspect(aspect))
        {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("plain aspect locator basis should be ready"),
        };
    let field_value = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::Value(AspectValueLocator::struct_field(field.clone())),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("field value locator basis should be ready"),
    };
    let plain_field = match prepare_locator_for_canonical_basis(
        version(),
        CanonicalLocatorInput::AspectField(field),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("plain aspect field locator basis should be ready"),
    };

    assert_ne!(
        whole_value.payload().entries(),
        plain_aspect.payload().entries()
    );
    assert_ne!(
        field_value.payload().entries(),
        plain_field.payload().entries()
    );
    assert!(whole_value.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("value.kind".into())
            && entry.value() == &CanonicalBasisValue::ExactText("whole_aspect".into())
    }));
    assert!(field_value.payload().entries().iter().any(|entry| {
        entry.locus() == &CanonicalBasisLocus::Named("value.kind".into())
            && entry.value() == &CanonicalBasisValue::ExactText("struct_field".into())
    }));
}
