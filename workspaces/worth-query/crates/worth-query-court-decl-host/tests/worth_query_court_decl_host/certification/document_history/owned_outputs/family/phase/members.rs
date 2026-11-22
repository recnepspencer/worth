//! Member targets and effects of the member-writing fixture producers.
// A decision that refuses in its domain on what it read (PR C §1.7).
macro_rules! producer_decision_refusal {
    (GuardedInputInitial, $value:ident) => { producer_decision_refusal!(@guarded $value) };
    (GuardedInputPreserve, $value:ident) => { producer_decision_refusal!(@guarded $value) };
    (GuardedMemberInitial, $value:ident) => { producer_decision_refusal!(@guarded $value) };
    (GuardedMemberPreserve, $value:ident) => { producer_decision_refusal!(@guarded $value) };
    (@guarded $value:ident) => {
        if $value >= 100 { Some(Denial::PreserveMissingOutput) } else if $value % 2 == 1 { Some(Denial::DerivedValueOutOfRange) } else { None }
    };
    ($operation:ident, $value:ident) => { None::<Denial> };
}
macro_rules! producer_member_target {
    (CrossConvergeInitial, $reader:ident, $input:ident) => { producer_member_target!(@cross $reader, $input) };
    (CrossConvergePreserve, $reader:ident, $input:ident) => { producer_member_target!(@cross $reader, $input) };
    (CrossOscillateInitial, $reader:ident, $input:ident) => { producer_member_target!(@cross $reader, $input) };
    (CrossOscillatePreserve, $reader:ident, $input:ident) => { producer_member_target!(@cross $reader, $input) };
    (@cross $reader:ident, $input:ident) => {{
        let other = if $input.0.key == "p" { "x:r" } else { "x:p" };
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), other.to_owned())?;
        $reader.field(&member, MemberValue::at("MemberValue"))?;
        Some($reader.mutation_target(&member)?)
    }};
    ($operation:ident, $reader:ident, $input:ident) => { producer_member_target!($operation, $reader) };
    (ConvergentPreserve, $reader:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        Some($reader.mutation_target(&member)?)
    }};
    (SelfWriteInitial, $reader:ident) => { producer_member_target!(@write $reader) };
    (SelfWritePreserve, $reader:ident) => { producer_member_target!(@write $reader) };
    (DecisionWriteInitial, $reader:ident) => { producer_member_target!(@write $reader) };
    (DecisionWritePreserve, $reader:ident) => { producer_member_target!(@write $reader) };
    (@write $reader:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        $reader.field(&member, MemberValue::at("MemberValue"))?;
        Some($reader.mutation_target(&member)?)
    }};
    ($operation:ident, $reader:ident) => { None };
}
macro_rules! producer_member_effect {
    (CrossConvergeInitial, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@cross $writer, $decision, ($decision.member_value + 1).min(3)) };
    (CrossConvergePreserve, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@cross $writer, $decision, ($decision.member_value + 1).min(3)) };
    (CrossOscillateInitial, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@cross $writer, $decision, $decision.member_value + 1) };
    (CrossOscillatePreserve, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@cross $writer, $decision, $decision.member_value + 1) };
    (@cross $writer:ident, $decision:ident, $value:expr) => {{
        let member = $writer.projected_entity($decision.member_target.as_ref().expect("actual completed cross decision"))?;
        $writer.write_field(&member, MemberValue::at("MemberValue"), $value)?;
    }};
    (ConvergentPreserve, $writer:ident, $decision:ident, $input:ident) => {{
        let member = $writer.projected_entity($decision.member_target.as_ref().expect("completed calibration decision"))?;
        $writer.write_field(&member, CalibrationReady::at("Ready"), 1)?;
    }};
    (SelfWriteInitial, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@write $writer, $decision, $input) };
    (SelfWritePreserve, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@write $writer, $decision, $input) };
    (DecisionWriteInitial, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@write $writer, $decision, $input) };
    (DecisionWritePreserve, $writer:ident, $decision:ident, $input:ident) => { producer_member_effect!(@write $writer, $decision, $input) };
    (@write $writer:ident, $decision:ident, $input:ident) => {{
        let member = $writer.projected_entity($decision.member_target.as_ref().expect("actual completed member decision"))?;
        $writer.write_field(&member, MemberValue::at("MemberValue"), $input.0.value + 1)?;
    }};
    // A producer with no member effect decided no member target.
    ($operation:ident, $writer:ident, $decision:ident, $input:ident) => { assert!($decision.member_target.is_none()) };
}
