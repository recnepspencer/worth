use super::fixture::real_fixture;
use crate::retention::{ComponentBasisDependencyClass, ExactComponentPinRequest};

#[test]
fn admitted_basis_pin_and_unpin_copy_zero_encoding_bytes() {
    let fixture = real_fixture(4, 4);
    let request = ExactComponentPinRequest::signal(
        &fixture.basis,
        ComponentBasisDependencyClass::ActivePublicationAttempt,
    );
    drop(fixture.owner.issue_component(request).unwrap());
    let key = request.key();
    let original = {
        let state = fixture.owner.lock();
        match &state.entries.get(&key).unwrap().basis_order {
            super::super::ComponentBasisOrder::Signal { reference } => reference.as_ptr(),
            _ => unreachable!(),
        }
    };
    let pin = fixture.owner.issue_component(request).unwrap();
    let mut copied = {
        let state = fixture.owner.lock();
        let reference = match &state.entries.get(&key).unwrap().basis_order {
            super::super::ComponentBasisOrder::Signal { reference } => reference,
            _ => unreachable!(),
        };
        let admission_copy = if reference.as_ptr() == original {
            0
        } else {
            reference.len()
        };
        admission_copy + state.entries.copied_encoding_bytes(&key)
    };
    drop(pin);
    copied += fixture.owner.lock().entries.copied_encoding_bytes(&key);
    assert_eq!(copied, 0, "encoding bytes copied on pin and unpin");
}

#[test]
fn admitted_basis_pair_reuses_its_encoding_on_pin_and_unpin() {
    let fixture = real_fixture(4, 4);
    let request = ExactComponentPinRequest::signal(
        &fixture.basis,
        ComponentBasisDependencyClass::ProductBranchHead,
    );
    drop(fixture.owner.issue_product_head(&fixture.basis).unwrap());
    let key = request.key();
    let snapshot = || {
        let state = fixture.owner.lock();
        let super::super::ComponentBasisOrder::Signal { reference } =
            &state.entries.get(&key).unwrap().basis_order
        else {
            unreachable!()
        };
        (
            reference.as_ptr(),
            reference.len(),
            state.entries.copied_encoding_bytes(&key),
        )
    };
    let (original, _, _) = snapshot();
    let pin = fixture.owner.issue_product_head(&fixture.basis).unwrap();
    let (pinned, bytes, indexed_copy) = snapshot();
    drop(pin);
    let (unpinned, _, unpin_copy) = snapshot();
    let copied = usize::from(original != pinned) * bytes
        + usize::from(pinned != unpinned) * bytes
        + indexed_copy
        + unpin_copy;
    assert_eq!(copied, 0, "encoding bytes copied on paired pin and unpin");
}

#[test]
fn equal_descriptions_visit_the_earlier_owner_issued_lease_first() {
    let fixture = real_fixture(4, 4);
    let request = |signal| {
        let dependency = ComponentBasisDependencyClass::ActivePublicationAttempt;
        if signal {
            ExactComponentPinRequest::signal(&fixture.basis, dependency)
        } else {
            ExactComponentPinRequest::relational(&fixture.basis, dependency)
        }
    };
    drop(fixture.owner.issue_component(request(false)).unwrap());
    drop(fixture.owner.issue_component(request(true)).unwrap());
    let first = request(false).key();
    let second = request(true).key();
    let mut state = fixture.owner.lock();
    let mut earlier = state.entries.remove(&first).unwrap();
    let later = state.entries.remove(&second).unwrap();
    assert!(earlier.lease_identity < later.lease_identity);
    // Descriptions are comparison data here; both lease identities came through
    // the real owner. Equal descriptions must preserve both lease positions.
    earlier.basis_order = later.basis_order.clone();
    let mut pins = super::super::RetainedComponentPins::default();
    pins.insert(second.clone(), later);
    pins.insert(first.clone(), earlier);
    assert_eq!(
        pins.by_declared_basis()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>(),
        [first, second]
    );
}
