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
