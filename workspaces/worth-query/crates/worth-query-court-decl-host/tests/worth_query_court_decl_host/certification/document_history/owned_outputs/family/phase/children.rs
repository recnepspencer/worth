//! Child records a fixture producer owns beside its one derived output.
macro_rules! producer_role_families {
    (ChildDropInitial) => { producer_role_families!(@kids) };
    (ChildDropPreserve) => { producer_role_families!(@kids) };
    (KidLinkerInitial) => { producer_role_families!(@kids) };
    (KidLinkerPreserve) => { producer_role_families!(@kids) };
    (RingDropInitial) => { producer_role_families!(@ring) };
    (RingDropPreserve) => { producer_role_families!(@ring) };
    (RingPeerInitial) => { producer_role_families!(@ring_peer) };
    (RingPeerPreserve) => { producer_role_families!(@ring_peer) };
    (@ring_peer) => {
        &[<super::Own<Outputs> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    };
    (@ring) => {
        &[<super::Ring<Outputs> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    };
    (@kids) => {
        &[<super::Kids<Outputs> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    };
    ($other:ident) => {
        &[]
    };
}
// Every child of its own the prior output links to (`ChildOfFirst`), with its
// presence: a decision may only delete a record whose presence it read.
// Another producer's child on the same list (`kid-linker`) is not its to drop.
macro_rules! child_drop_capture {
    ($reader:ident, $prior:ident) => {{
        let mut kids = Vec::new();
        if let Some(output) = &$prior {
            for link in $reader.relations_from(ChildOfFirst::reference(), output).map_err(HandlerExecutionDenial::new)? {
                $reader.entity(link.to(), ChildOutput::reference()).map_err(HandlerExecutionDenial::new)?;
                if $reader.field(link.to(), ChildKey::at("ChildKey"))?.is_some_and(|key| key.starts_with("kid:")) {
                    kids.push($reader.mutation_target(link.to())?);
                }
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
            create_kid!($writer, $decision, $input, super::child_key(&$input.0), &$output);
        }
    };
}
// Output `first:a`, which `child-drop(a)` derives: the place for this
// producer's one child, once it has a prior output of its own and has not
// linked the child yet. So the link lands in the edit after the one that
// created `first:a`, behind `child-drop(a)`'s Preserve head round.
macro_rules! kid_linker_capture {
    ($reader:ident, $prior:ident) => {{
        let linked = $reader.resolve_optional_entity(ChildKey::at("ChildKey"), super::LINKED_KID.to_owned())?.is_some();
        if $prior.is_some() && !linked {
            let first = $reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
            $reader.entity(&first, FirstOutput::reference()).map_err(HandlerExecutionDenial::new)?;
            Some($reader.mutation_target(&first)?)
        } else {
            None
        }
    }};
}
macro_rules! kid_linker_publish {
    ($writer:ident, $decision:ident, $input:ident) => {
        if let Some(target) = &$decision.input_member {
            let first = $writer.projected_entity(target)?;
            create_kid!($writer, $decision, $input, super::LINKED_KID.to_owned(), &first);
        }
    };
}
// One new child under `$parent`, a member of the producer's `Kids`.
macro_rules! create_kid {
    ($writer:ident, $decision:ident, $input:ident, $key:expr, $parent:expr) => {{
        let owner = $writer.projected_entity(&$decision.source)?;
        let key: String = $key;
        let kid = $writer.create_entity_in_context(&owner, ChildOutput::reference(), WorthQueryApplicationEntityKey::new(&key).unwrap())?;
        $writer.initialize_field(&kid, ChildKey::at("ChildKey"), key.clone())?;
        $writer.initialize_field(&kid, ChildValue::at("ChildValue"), 0)?;
        $writer.link(ChildOfFirst::reference(), format!("of:{key}"), $parent, &kid)?;
        $writer.create_member::<super::Kids<Outputs>>(&$input.0.key, &kid)?;
    }};
}
