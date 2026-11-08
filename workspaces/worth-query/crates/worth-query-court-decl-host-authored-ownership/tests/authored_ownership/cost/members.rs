//! A genuine installed producer owns N independent members and replaces one.
use super::*;
use coupled::producer::Binding as Preserve;
use sheet_boundary::{InitialNodes, Number, Occurrence, Profile, RetiredNodes, SurvivingNodes};

pub struct Members<const N: usize>;
impl<const N: usize> Profile for Members<N> {
    type Relations = ();
    const LINK_BOUNDARY: bool = false;
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, false>(
            PhantomData,
        ))?
        .build()
    }
    type Decision<S: Scenario> = Vec<(
        u64,
        WorthQueryInvariantMutationTarget<S, Occurrence<S::Boundary>>,
    )>;
    fn initial<S: Scenario>(
        w: &mut CandidateWriter<'_, S, producer::Binding<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        for index in 0..N {
            let node = w.create_entity_in_context(
                source,
                Occurrence::<S::Boundary>::reference(),
                WorthQueryApplicationEntityKey::new(format!("member-{index}")).unwrap(),
            )?;
            w.initialize_field(&node, Number::<S::Boundary>::at("Number"), index as u64)?;
            w.create_member::<InitialNodes<S>>(&index.to_string(), &node)?;
        }
        Ok(())
    }
    fn read<S: Scenario>(
        r: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    ) -> Result<Option<Self::Decision<S>>, HandlerExecutionDenial> {
        let members = match r
            .prior_output_family_if_present::<Preserve<S>, SurvivingNodes<S>>()
            .map_err(HandlerExecutionDenial::new)?
        {
            Some(nodes) => nodes
                .into_iter()
                .map(|node| node.into_identity())
                .collect::<Vec<_>>(),
            None => r
                .prior_output_family::<producer::Binding<S>, InitialNodes<S>>()
                .map_err(HandlerExecutionDenial::new)?
                .into_iter()
                .map(|node| node.into_identity())
                .collect(),
        };
        assert_eq!(
            members.len(),
            N,
            "complete desired roster equals replace-all's member count"
        );
        let mut targets = Vec::new();
        for node in members {
            let index = r
                .field(&node, Number::<S::Boundary>::at("Number"))?
                .unwrap();
            targets.push((index, r.mutation_target(&node)?));
        }
        targets.sort_by_key(|(index, _)| *index);
        let numbers = targets.iter().map(|(index, _)| *index).collect::<Vec<_>>();
        assert_eq!(
            numbers,
            (numbers[0]..numbers[0] + N as u64).collect::<Vec<_>>(),
            "membership equals independent sliding-window oracle"
        );
        Ok(Some(targets))
    }
    fn trim<S: Scenario>(
        w: &mut CandidateWriter<'_, S, Preserve<S>>,
        decision: Option<Self::Decision<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let nodes = decision.unwrap();
        let next = nodes.last().unwrap().0 + 1;
        for (ordinal, (index, target)) in nodes.into_iter().enumerate() {
            let node = w.projected_entity(&target)?;
            if ordinal == 0 {
                w.delete_entity(Occurrence::<S::Boundary>::reference(), &node)?;
                w.retire_member::<RetiredNodes<S>>(&index.to_string(), &node)?;
            } else {
                w.preserve_member::<SurvivingNodes<S>>(&index.to_string(), &node)?;
            }
        }
        let node = w.create_entity_in_context(
            source,
            Occurrence::<S::Boundary>::reference(),
            WorthQueryApplicationEntityKey::new(format!("member-{next}")).unwrap(),
        )?;
        w.initialize_field(&node, Number::<S::Boundary>::at("Number"), next)?;
        w.create_member::<SurvivingNodes<S>>(&next.to_string(), &node)?;
        Ok(())
    }
}
