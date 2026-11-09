//! Admission work equals the values actually supplied or produced, not their
//! predecessor copies. The codec observer does not affect the value's bits.
use super::*;
use std::{cell::Cell, rc::Rc};
#[derive(Clone)]
struct Value {
    bits: u64,
    encodings: Rc<Cell<u64>>,
}
impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        0
    } // Rc clone shares observer allocation.
}
impl CanonicalBits for Value {
    fn canonical_len(&self) -> Option<usize> {
        self.bits.canonical_len()
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        self.encodings.set(self.encodings.get() + 1);
        self.bits.visit_canonical_bits(visit)
    }
}
#[test]
fn canonical_admission_equals_new_inputs_and_actual_combine_outputs() {
    for count in [64, 128] {
        let encodings = Rc::new(Cell::new(0));
        let combines = Rc::new(Cell::new(0));
        let value = |bits| Value {
            bits,
            encodings: encodings.clone(),
        };
        let combine_count = combines.clone();
        let combine = move |a: &Value, b: &Value| {
            combine_count.set(combine_count.get() + 1);
            Value {
                bits: a.bits + b.bits,
                encodings: a.encodings.clone(),
            }
        };
        let identities = (1..=count).map(PartitionIdentity::new).collect::<Vec<_>>();
        let entries = identities
            .iter()
            .map(|&id| (id, value(id.value())))
            .collect();
        let (mut tree, _) = ReductionTree::try_from_declared(
            ReductionPlan::try_from_sorted_unique(identities).unwrap(),
            entries,
            value(0),
            combine,
        )
        .unwrap();
        let prior = tree.clone();
        for incoming in [count + 1, count + 1] {
            encodings.set(0);
            combines.set(0);
            let inputs = vec![(PartitionIdentity::new(count), value(incoming))];
            let supplied = inputs.len() as u64;
            for (id, input) in inputs {
                tree.update(id, input).unwrap();
            }
            assert_eq!(
                encodings.get(),
                combines.get() + supplied,
                "each new input and actual combine output is admitted once"
            );
            assert_eq!(tree.result().bits, (1..count).sum::<u64>() + incoming);
        }
        encodings.set(0);
        combines.set(0);
        tree.delete(PartitionIdentity::new(1)).unwrap();
        assert_eq!(
            encodings.get(),
            combines.get(),
            "deletion carries all surviving leaf proofs"
        );
        assert_eq!(prior.result().bits, (1..=count).sum::<u64>());
    }
}
#[test]
fn a_new_ceiling_checks_carried_encoding_and_cloned_payload_without_reencoding() {
    let id = PartitionIdentity::new(1);
    let mut input = Vec::with_capacity(64);
    input.push(7u64);
    let (mut tree, _) = ReductionTree::try_from_declared(
        ReductionPlan::try_from_sorted_unique(vec![id]).unwrap(),
        vec![(id, input)],
        Vec::<u64>::new(),
        |a, b| a.iter().chain(b).copied().collect(),
    )
    .unwrap();
    let before = tree.result().clone();
    let denied = tree.update_checked(id, vec![8], size_of::<u64>() as u64, || Ok::<_, ()>(()));
    assert!(matches!(
        denied.unwrap_err().reason,
        ReductionRunStop::ResultCapacityExceeded
    ));
    assert_eq!(tree.result(), &before);
}
