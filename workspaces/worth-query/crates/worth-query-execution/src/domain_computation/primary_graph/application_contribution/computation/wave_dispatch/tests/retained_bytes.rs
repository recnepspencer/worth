//! Native heap inputs cross the production wave without a retained-byte invoice.
use super::*;

#[test]
fn heap_input_runs_without_a_payload_invoice() {
    let mut text = String::with_capacity(257);
    text.push_str("captured input");
    let callback = move |member: WorthQueryMemberExecution<'_>| {
        let (_, poll) = member.into_parts();
        poll().unwrap();
        text.len()
    };
    let prepared = PreparedCompute::new(callback);
    let mut requests = Vec::with_capacity(3);
    requests.push(prepared);
    let member = PreparedMember { requests };
    let (mut settled, _) = run(None, vec![member]);
    let mut slots = settled.pop().unwrap().unwrap().into_slots();
    assert_eq!(slots.pop().unwrap().into_value::<usize>(), 14);
}
