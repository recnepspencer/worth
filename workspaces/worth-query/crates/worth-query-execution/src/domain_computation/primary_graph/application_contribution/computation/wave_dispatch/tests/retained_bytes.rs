//! The production derived envelope delegates erased input storage to its owner.
use super::*;

#[test]
fn heap_input_is_counted_before_the_real_wave_runs() {
    let mut text = String::with_capacity(257);
    text.push_str("captured input");
    let owned = text.capacity() as u64;
    let callback = move |member: WorthQueryMemberExecution<'_>| {
        let (_, mut charge) = member.into_parts();
        charge.as_mut().unwrap()(1).unwrap();
        text.len()
    };
    // PreparedCompute adds its erased callback allocation to this owned heap.
    let prepared = PreparedCompute::new(owned, 0, callback, |_| 0);
    let mut requests = Vec::with_capacity(3);
    requests.push(prepared);
    let member = PreparedMember { requests };
    let minimum = 3 * size_of::<PreparedCompute>() as u64 + owned;
    assert!(member.additional_charged_bytes() >= minimum);
    let (mut settled, report) = run(None, vec![member]);
    let mut slots = settled.pop().unwrap().unwrap().into_slots();
    assert_eq!(slots.pop().unwrap().into_value::<usize>(), 14);
}
