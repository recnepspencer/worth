# worth-store-layout-indexes

Owns Roadmap 2 S.8 and supports Roadmap 1 layout/index-heavy milestones:
declared layout families, index format versions, secondary-index rebuild
contracts, access-path counters, read amplification, and write amplification.

Every artifact family should have an explicit layout strategy instead of
silently inheriting a generic storage shape.

Baseline B-tree lookup returns `StableBTreeLookupExecution`. Its
`read_plan_completion()` reports the completed local plan's root, footprint,
and planning counters. Byte-touch and lookup costs remain in the lookup's
`counter_receipt()` and execution view. Plan completion is not a Store root
lease or evidence that Store byte guards executed; callers cannot promote it
to `StablePhysicalReadReceipt` or use it to authorize reclamation.
