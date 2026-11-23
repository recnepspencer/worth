//! Child records a fixture producer owns beside its one derived output.
macro_rules! producer_role_families {
    (ChildDropInitial) => { producer_role_families!(@kids) };
    (ChildDropPreserve) => { producer_role_families!(@kids) };
    (@kids) => {
        &[<super::Kids<Outputs> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    };
    ($other:ident) => {
        &[]
    };
}
// Every child the prior output links to (`ChildOfFirst`), with its presence:
// a decision may only delete a record whose presence it read.
macro_rules! child_drop_capture {
    ($reader:ident, $prior:ident) => {{
        let mut kids = Vec::new();
        if let Some(output) = &$prior {
            for link in $reader.relations_from(ChildOfFirst::reference(), output).map_err(HandlerExecutionDenial::new)? {
                $reader.entity(link.to(), ChildOutput::reference()).map_err(HandlerExecutionDenial::new)?;
                kids.push($reader.mutation_target(link.to())?);
            }
        }
        kids
    }};
}
// Preserve drops every child it found; Initial creates the one child.
macro_rules! child_drop_publish {
    ($writer:ident, $decision:ident, $output:ident, $input:ident) => {
        for target in &$decision.input_member {
            let kid = $writer.projected_entity(target)?;
            $writer.unlink(ChildOfFirst::reference(), &$output, &kid)?;
            $writer.delete_entity(ChildOutput::reference(), &kid)?;
            $writer.retire_member::<super::Kids<Outputs>>(&$input.0.key, &kid)?;
        }
        if $decision.output.is_none() {
            let owner = $writer.projected_entity(&$decision.source)?;
            let key = super::child_key(&$input.0);
            let kid = $writer.create_entity_in_context(&owner, ChildOutput::reference(), WorthQueryApplicationEntityKey::new(&key).unwrap())?;
            $writer.initialize_field(&kid, ChildKey::at("ChildKey"), key.clone())?;
            $writer.initialize_field(&kid, ChildValue::at("ChildValue"), 0)?;
            $writer.link(ChildOfFirst::reference(), format!("of:{key}"), &$output, &kid)?;
            $writer.create_member::<super::Kids<Outputs>>(&$input.0.key, &kid)?;
        }
    };
}
