# Partial Home index precondition regression

Earlier logs d12-home-final-reopen-and-bounded-courts.log and
d12-host-filesystem-reopen-courts.log pass the unchanged initializer-refusal
court. funding-finish-host-bounded-reopen-kill-ack.log fails it on missing
application-principal.IdentityBinding after the close upkeep change.

The shared upkeep correction makes the zero-write partial Home publish a real
checkpoint at close; the earlier unavailable worker retained only its tails.
This changes which accelerator is read, not which Query installation events
were committed. The actual Native history catalog supplies the recovered publication frontier.
RecoveryOutcome is only a publicly constructible summary and grants no proof. At zero writes, an empty recovered index inventory is
lawful: initialization failed before the seed publication that would carry
the Query catalog in a canonical input. Requiring a full installed catalog in
that state is the wrong precondition. Do not register fresh definitions.

Minimal owner repair: existing.rs uses a named precondition for recovered
index binding: either a real recovered publication exists, or any definitions
were recovered. In either case require exact recovered definitions as today.
Only the actual zero-publication/empty-inventory state has no indexes to bind.
Continue existing family validation and original World selection unchanged.
The existing World origin refusal and callback/held-lease assertions remain
unchanged. No checkpoint is suppressed, and no D8 phase is moved.

Validate the unchanged initializer-refusal court plus selected/no-tail and
journal-only controls on the actual compiled product. Any missing definition
in a published Home still fails the exact recovered-definition contract.

Independent review caught the summary/owner distinction before qualification.
The guard now consumes native.history().latest_catalog_commit_identity(),
together with the actual definition lookup inventory.
