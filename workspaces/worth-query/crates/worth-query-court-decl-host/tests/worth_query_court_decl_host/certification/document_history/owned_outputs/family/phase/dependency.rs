//! What each fixture producer's decision reads beyond its source.
macro_rules! producer_decision_dependency {
    (CycleParentInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cycle_parent $reader) };
    (CycleConsumerInitial, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::cycle_parent::Family)
    }};
    (CycleParentPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cycle_parent $reader) };
    (CycleConsumerPreserve, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::cycle_parent::Family)
    }};
    // min(m, 5); once m > 5, also read consumer `cycle-consumer(c)`.
    (@cycle_parent $reader:ident) => {{
        let member = producer_decision_dependency!(@member_read $reader);
        if member > 5 {
            let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "c".to_owned())?;
            producer_decision_dependency!(@current_output $reader, model, super::super::cycle_consumer::Family);
        }
        member.min(5)
    }};
    (GuardedCurrentInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@saturated_current $reader) };
    (GuardedCurrentPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@saturated_current $reader) };
    // Whether `saturated-input(a)` publishes a current output for model `a`.
    (@saturated_current $reader:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        match $reader.current_output::<super::super::saturated_input::Family, Model>(&model)? {
            WorthQueryCurrentOutputSelection::Unique(_) => 1,
            _ => 0,
        }
    }};
    (SaturatedConsumerInitial, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::saturated_input::Family)
    }};
    (CappedReaderInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input).min(1) };
    (CappedConsumerInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@current_output $reader, $source, super::super::capped_reader::Family) };
    (SaturatedConsumerPreserve, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::saturated_input::Family)
    }};
    (CappedReaderPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input).min(1) };
    (CappedConsumerPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@current_output $reader, $source, super::super::capped_reader::Family) };
    // The value of the output `$family` currently publishes for `$producer`.
    (@current_output $reader:ident, $producer:ident, $family:ty) => {{
        match $reader.current_output::<$family, Model>(&$producer)? {
            WorthQueryCurrentOutputSelection::Unique(output) => $reader
                .field(&output, FirstValue::at("FirstValue"))?
                .expect("current output has its value"),
            _ => panic!("the consumed output is current and unique"),
        }
    }};
    (SelfWriteInitial, $reader:ident, $source:ident, $input:ident) => {{
        let first = $reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
        $reader.field(&first, FirstValue::at("FirstValue"))?.expect("derived first:a has its value")
    }};
    (SelfWritePreserve, $reader:ident, $source:ident, $input:ident) => {{
        let first = $reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
        $reader.field(&first, FirstValue::at("FirstValue"))?.expect("derived first:a has its value")
    }};
    (CrossReaderInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input) };
    (CrossReaderPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input) };
    (CrossConvergeInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossConvergePreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossOscillateInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossOscillatePreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (@cross_member $reader:ident, $input:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), format!("x:{}", $input.0.key))?;
        $reader.field(&member, MemberValue::at("MemberValue"))?.expect("seeded cross member has its value")
    }};
    (@cross_supplier $reader:ident, $input:ident) => {{
        let supplier = $reader.resolve_entity(FirstKey::at("FirstKey"), format!("cross:{}", $input.0.key))?;
        $reader.field(&supplier, FirstValue::at("FirstValue"))?.expect("derived cross reader has its value")
    }};
    // Every other family reads no demand key.
    ($operation:ident, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!($operation, $reader, $source) };
    (ConvergentPreserve, $reader:ident, $source:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        $reader.field(&member, CalibrationReady::at("Ready"))?.unwrap_or(0)
    }};
    (OrdinaryInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (OrdinaryInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (DecisionInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (SelfRelationInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@self_relation $reader, $source) };
    (SelfRelationPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@self_relation $reader, $source) };
    // Member `m`, and every output the source model owns (`FirstOf`, read
    // at the current image), each of which must have its value.
    (@self_relation $reader:ident, $source:ident) => {{
        let m = producer_decision_dependency!(@member_read $reader);
        for link in $reader.relations_to(FirstOf::reference(), &$source).map_err(HandlerExecutionDenial::new)? {
            if $reader.field(link.from(), FirstValue::at("FirstValue"))?.is_none() {
                // A reached output without its value: a required record is missing.
                $reader.resolve_entity(FirstKey::at("FirstKey"), "self-relation:missing".to_owned())?;
            }
        }
        m
    }};
    (DecisionInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (ParityInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (ParityInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (SaturatedInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader).min(5) };
    (SaturatedInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader).min(5) };
    (GuardedInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@model_b $reader) };
    (GuardedInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@model_b $reader) };
    (GuardedMemberInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (GuardedFirstInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@first_b_presence $reader) };
    (GuardedFirstPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@first_b_presence $reader) };
    // Whether output `first:b` exists: an optional indexed lookup.
    (@first_b_presence $reader:ident) => {
        u64::from($reader.resolve_optional_entity(FirstKey::at("FirstKey"), "first:b".to_owned())?.is_some())
    };
    (GuardedPresenceInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@saturated_presence $reader) };
    (GuardedPresencePreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@saturated_presence $reader) };
    // Whether output `saturated:a` exists: an optional indexed lookup, a source fact.
    (@saturated_presence $reader:ident) => {
        u64::from($reader.resolve_optional_entity(FirstKey::at("FirstKey"), "saturated:a".to_owned())?.is_some())
    };
    (GuardedMemberPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (@model_b $reader:ident) => {{
        let b = $reader.resolve_entity(ModelKey::at("ModelKey"), "b".to_owned())?;
        $reader.field(&b, ModelValue::at("ModelValue"))?.expect("model b has its value")
    }};
    (@member_read $reader:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        $reader.field(&member, MemberValue::at("MemberValue"))?.expect("seeded decision member has its required value")
    }};
    ($operation:ident, $reader:ident, $source:ident) => { 0 };
}
