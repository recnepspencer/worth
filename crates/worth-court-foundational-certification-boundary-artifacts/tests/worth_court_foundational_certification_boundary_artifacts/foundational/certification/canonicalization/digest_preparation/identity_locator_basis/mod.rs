#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::digest_preparation::identity_locator_basis::*;
use {worth_foundational::diagnostic_mask_locator_canonical_basis_entries, worth_foundational::identity_canonical_basis_entries, worth_foundational::locator_canonical_basis_entries, worth_foundational::mutation_mask_locator_canonical_basis_entries, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_identity_for_canonical_basis, worth_foundational::prepare_locator_for_canonical_basis, worth_foundational::projection_mask_locator_canonical_basis_entries, worth_foundational::AspectFieldLocator, worth_foundational::AspectKey, worth_foundational::AspectLocator, worth_foundational::AspectMask, worth_foundational::AspectValueLocator, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryEpoch, worth_foundational::BoundaryHandle, worth_foundational::BoundaryMismatchLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalFieldPath, worth_foundational::CanonicalIdentityInput, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalLocatorInput, worth_foundational::CanonicalizationRuleVersion, worth_foundational::DiagnosticMask, worth_foundational::EquivalenceBasisId, worth_foundational::FieldKey, worth_foundational::LocatorAuthority, worth_foundational::MutationMask, worth_foundational::ProjectionMask};
use {worth_proof::TransitionOutcome};

#[test]
fn identity_basis_keeps_equal_storage_categories_distinct() {
    let handle = match prepare_identity_for_canonical_basis(
        version(),
        worth_foundational::CanonicalIdentityInput::BoundaryHandle(worth_foundational::BoundaryHandle::new(9)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("handle identity basis should be ready"),
    };
    let epoch = match prepare_identity_for_canonical_basis(
        version(),
        worth_foundational::CanonicalIdentityInput::BoundaryEpoch(worth_foundational::BoundaryEpoch::new(9)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("epoch identity basis should be ready"),
    };
    let equivalence = match prepare_identity_for_canonical_basis(
        version(),
        worth_foundational::CanonicalIdentityInput::EquivalenceBasis(worth_foundational::EquivalenceBasisId::new(9)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("equivalence identity basis should be ready"),
    };

    assert_eq!(handle.payload().domain(), worth_foundational::CanonicalBasisDomain::Identity);
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
        worth_foundational::BoundaryArtifactLocator::new(worth_foundational::BoundaryArtifactId::new(11), worth_foundational::BoundaryArtifactField::Basis);
    let source = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Source(worth_foundational::BoundarySourceLocator::boundary_artifact(artifact)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("source locator basis should be ready"),
    };
    let mismatch = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Mismatch(worth_foundational::BoundaryMismatchLocator::boundary_artifact(artifact)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("mismatch locator basis should be ready"),
    };

    assert_eq!(source.payload().domain(), worth_foundational::CanonicalBasisDomain::Locator);
    assert_ne!(source.payload().entries(), mismatch.payload().entries());
    assert!(source.payload().entries().iter().any(|entry| entry.locus()
        == &worth_foundational::CanonicalBasisLocus::Named("source.boundary_artifact.field".into())));
    assert!(mismatch
        .payload()
        .entries()
        .iter()
        .any(|entry| entry.locus()
            == &worth_foundational::CanonicalBasisLocus::Named("mismatch.boundary_artifact.field".into())));
    assert_eq!(
        locator_canonical_basis_entries(&source),
        source.payload().entries()
    );
}

#[test]
fn aspect_field_locator_basis_uses_semantic_path_not_debug_shape() {
    let locator = worth_foundational::AspectFieldLocator::new(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
        worth_foundational::CanonicalFieldPath::new([field("body"), field("plain")]).expect("nonempty path"),
    );
    let ready = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::AspectField(locator),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("field locator basis should be ready"),
    };

    assert!(ready.payload().entries().iter().any(|entry| {
        entry.kind() == worth_foundational::CanonicalBasisEntryKind::Locator
            && entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("aspect_field.field_path".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("body.plain".to_string().into())
    }));
}

#[test]
fn mask_locator_basis_preserves_mode_and_ordered_paths() {
    let projection = AspectMask::<ProjectionMask>::new([path("zeta"), path("alpha")]);
    let mutation = AspectMask::<MutationMask>::new([path("zeta"), path("alpha")]);
    let diagnostic = AspectMask::<DiagnosticMask>::new([path("zeta"), path("alpha")]);

    let projection_entries = projection_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::projection(
            worth_foundational::LocatorAuthority::Projected,
            key("task.summary"),
            &projection,
        ),
    );
    let mutation_entries = mutation_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::mutation(
            worth_foundational::LocatorAuthority::Projected,
            key("task.summary"),
            &mutation,
        ),
    );
    let diagnostic_entries = diagnostic_mask_locator_canonical_basis_entries(
        &worth_foundational::AspectMaskLocator::diagnostic(
            worth_foundational::LocatorAuthority::Projected,
            key("task.summary"),
            &diagnostic,
        ),
    );

    assert_ne!(projection_entries, mutation_entries);
    assert_ne!(mutation_entries, diagnostic_entries);

    let ready = match prepare_canonical_basis_sequence(
        version(),
        worth_foundational::CanonicalBasisDomain::Locator,
        projection_entries,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("mask locator entries should form a ready locator basis"),
    };

    assert_eq!(ready.payload().cost().entry_count(), 5);
    assert_eq!(
        ready.payload().entries()[3].value(),
        &worth_foundational::CanonicalBasisValue::ExactText("alpha".to_string().into())
    );
    assert_eq!(
        ready.payload().entries()[4].value(),
        &worth_foundational::CanonicalBasisValue::ExactText("zeta".to_string().into())
    );
}

#[test]
fn identity_and_locator_basis_do_not_collapse_adjacent_numeric_meanings() {
    let identity = match prepare_identity_for_canonical_basis(
        version(),
        worth_foundational::CanonicalIdentityInput::BoundaryArtifact(worth_foundational::BoundaryArtifactId::new(11)),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity basis should be ready"),
    };
    let locator = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::BoundaryArtifact(worth_foundational::BoundaryArtifactLocator::new(
            worth_foundational::BoundaryArtifactId::new(11),
            worth_foundational::BoundaryArtifactField::Payload,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("locator basis should be ready"),
    };

    assert_eq!(
        identity.payload().entries()[0].value(),
        &worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: 11,
        }
    );
    assert_eq!(identity.payload().domain(), worth_foundational::CanonicalBasisDomain::Identity);
    assert_eq!(locator.payload().domain(), worth_foundational::CanonicalBasisDomain::Locator);
    assert_ne!(
        identity.payload().entries()[0].locus(),
        locator.payload().entries()[0].locus()
    );
}

#[test]
fn aspect_locator_basis_preserves_authority_as_identity() {
    let authoritative = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::Authoritative,
            key("task.summary"),
        )),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("authoritative aspect locator basis should be ready"),
    };
    let projected = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::Projected,
            key("task.summary"),
        )),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("projected aspect locator basis should be ready"),
    };

    assert_ne!(
        authoritative.payload().entries(),
        projected.payload().entries()
    );
}

#[test]
fn value_locator_basis_is_distinct_from_plain_aspect_and_field_locator_basis() {
    let aspect = worth_foundational::AspectLocator::new(worth_foundational::LocatorAuthority::Authoritative, key("task.summary"));
    let field = worth_foundational::AspectFieldLocator::new(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
        worth_foundational::CanonicalFieldPath::new([field("body"), field("plain")]).expect("non-empty path"),
    );

    let whole_value = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Value(worth_foundational::AspectValueLocator::whole_aspect(aspect.clone())),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("whole-aspect value locator basis should be ready"),
    };
    let plain_aspect =
        match prepare_locator_for_canonical_basis(version(), worth_foundational::CanonicalLocatorInput::Aspect(aspect))
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("plain aspect locator basis should be ready"),
        };
    let field_value = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Value(worth_foundational::AspectValueLocator::struct_field(field.clone())),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("field value locator basis should be ready"),
    };
    let plain_field = match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::AspectField(field),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
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
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("value.kind".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("whole_aspect".into())
    }));
    assert!(field_value.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("value.kind".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("struct_field".into())
    }));
}
