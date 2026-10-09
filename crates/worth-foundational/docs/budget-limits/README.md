# Budget Limits

A refusal for want of budget and a refusal for damage mean opposite things. A
limit says nothing about the media or the input: the same work may pass under
a wider budget. Damage says the work cannot pass under any budget. Reporting
one as the other is a lie, so the two are told apart by type.

## The vocabulary

- `LimitDimension`: what ran out. Each budget owner implements it on its own
  enum, and names the sealed authority that alone may mint limits of it.
- `LimitCounts`: what the budget held when it refused. `observed` is at least
  what the work needed; `admitted` is what the budget allowed.
- `BudgetRefused`: the action an owner's `refuse` door records when its
  budget refuses.
- `ExhaustedLimit<D>`: the limit. Private fields; its one constructor,
  `refused`, consumes the owner's `Performed<BudgetRefused, D::Authority,
  LimitCounts>`.

## The law

A limit comes only from the budget that owns its dimension, with that budget's
counts. Each owner declares, in a leaf module (the declaring module's
descendants can mint too):

1. its dimension enum, implementing `LimitDimension`;
2. its sealed authority, with `worth_foundational::limit_authority!(pub Owner)`,
   the one way to declare a budget owner. It adds one private
   `Owner::refuse(dimension, counts)` door that records the refusal and mints
   the limit in one step;
3. the budget whose refuse path alone calls `Owner::refuse`, with the real
   observed and admitted counts.

Coherence admits one `LimitDimension` implementation per dimension, and the
orphan rule keeps it in the crate that declares the enum. So damage reported as
a limit does not compile, and a limit cannot be built from numbers no budget
produced.

No method relabels a limit's dimension. An owner that reports another owner's
limit as its own refuses through its own budget, with its own counts.

## What it does not check

A limit reported as damage still compiles: any `match` arm can return a damage
variant. The defense is review, made possible by damage vocabularies with no
catch-all variant, so every damage site names a specific kind.
