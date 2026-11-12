//! Recorded absence can acquire one audience; selected original meaning never changes.
use super::*;

#[test]
fn transport_binding_preserves_explicit_absence_then_one_original_audience_and_source() {
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let source = super::inbound::accepted().source;
    let mut binding = QueryTransportBinding {
        correlation: CanonicalDigestId::new([7; 32]),
        operation: "transfer".to_owned(),
        audience: None,
        source,
    };
    for audience in [None, Some("original-audience".to_owned())] {
        binding.audience = audience;
        let frame = ledger
            .prepare(
                QueryDispatchRecordIntent::Inbound(QueryInboundRecord::TransportBinding(
                    binding.clone(),
                )),
                &mut generations,
            )
            .unwrap();
        let decoded =
            QueryDispatchFrame::readmit(frame.bytes(), frame.bytes().len() as u64).unwrap();
        assert_eq!(frame, decoded);
        ledger.readmit(decoded).unwrap();
        assert_eq!(
            ledger.inbound.transport_binding(&binding.correlation),
            Some(&binding)
        );
    }
    let mut foreign_audience = binding.clone();
    foreign_audience.audience = Some("replacement-audience".to_owned());
    let mut absence = binding.clone();
    absence.audience = None;
    let mut foreign_source = binding.clone();
    foreign_source.source.native_commit += 1;
    let mut foreign_operation = binding;
    foreign_operation.operation.push_str("-different");
    for changed in [foreign_audience, absence, foreign_source, foreign_operation] {
        assert_eq!(
            ledger.prepare(
                QueryDispatchRecordIntent::Inbound(QueryInboundRecord::TransportBinding(changed)),
                &mut generations
            ),
            Err(Denial::MalformedFrame)
        );
    }
}
