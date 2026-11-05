# Hub map: `WorthQueryPrimaryGraphBootstrap`

- impl blocks: 12; methods: 25
- impl blocks by module: application_runtime 1, bootstrap 1, bootstrap::advancement_publication 1, bootstrap::checkpoint 1, bootstrap::managed_source_initialization 1, bootstrap::publication_target 1, bootstrap::seed_batches 1, bootstrap::truth_partition 1, bootstrap::unique_seed_values 1, handler 1, invariant_projection 1, typed_bootstrap 1

## Fields, by the modules whose methods touch them

| field | owner | methods | modules (methods touching) |
|---|---|---|---|
| `graph` | shared | 11 | typed_bootstrap 2, bootstrap 1, bootstrap::advancement_publication 1, bootstrap::managed_source_initialization 1, bootstrap::publication_target 1, bootstrap::seed_batches 1 |
| `entity_rows` | bootstrap::seed_batches | 4 | bootstrap::seed_batches 2, bootstrap::advancement_publication 1, typed_bootstrap 1 |
| `pending_entity_keys` | typed_bootstrap | 4 | typed_bootstrap 2, bootstrap 1, bootstrap::seed_batches 1 |
| `relation_rows` | bootstrap::seed_batches | 4 | bootstrap::seed_batches 2, bootstrap::advancement_publication 1, typed_bootstrap 1 |
| `rows` | bootstrap::seed_batches | 4 | bootstrap::seed_batches 2, bootstrap 1, bootstrap::advancement_publication 1 |
| `recovered_relational_authority` | shared | 3 | bootstrap 1, bootstrap::checkpoint 1, bootstrap::unique_seed_values 1 |
| `seed_batch_failed` | shared | 3 | bootstrap::advancement_publication 1, bootstrap::managed_source_initialization 1, bootstrap::seed_batches 1 |
| `committed_entity_count` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::seed_batches 1 |
| `committed_principal_count` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::seed_batches 1 |
| `committed_relation_count` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::seed_batches 1 |
| `entity_keys` | bootstrap | 2 | bootstrap 1, typed_bootstrap 1 |
| `last_seed_commit_id` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::seed_batches 1 |
| `pending_source_initialization` | bootstrap::managed_source_initialization | 2 | bootstrap::managed_source_initialization 2 |
| `program_activation_seed` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::seed_batches 1 |
| `recovered_publication` | bootstrap::advancement_publication | 2 | bootstrap::advancement_publication 1, bootstrap::managed_source_initialization 1 |
| `resource_support` | bootstrap | 2 | bootstrap 1, bootstrap::advancement_publication 1 |
| `runtime_authority` | bootstrap::publication_target | 2 | bootstrap::publication_target 1, invariant_projection 1 |
| `source_initializer` | bootstrap::managed_source_initialization | 2 | bootstrap::managed_source_initialization 2 |
| `expected_invariant_inventory_digest` | shared | 1 | bootstrap::publication_target 1 |
| `external_identities` | shared | 1 | bootstrap 1 |
| `installed_operations` | shared | 1 | invariant_projection 1 |
| `installed_packages` | shared | 1 | bootstrap 1 |
| `invariant_installation_receipt` | shared | 1 | bootstrap::publication_target 1 |
| `mutation_handlers` | shared | 1 | handler 1 |
| `principal_identities` | shared | 1 | bootstrap 1 |
| `principal_keys` | shared | 1 | bootstrap 1 |
| `program_activation` | shared | 1 | invariant_projection 1 |
| `recovered_checkpoint_restore_work` | shared | 1 | bootstrap 1 |
| `relation_keys` | shared | 1 | typed_bootstrap 1 |
| `unique_seed_values` | shared | 1 | bootstrap::unique_seed_values 1 |
| `value_bindings` | shared | 1 | invariant_projection 1 |

## Candidate subsystems (heaviest first)

- **bootstrap::advancement_publication**: 2 methods, 11 owned-field accesses; fields `committed_entity_count`, `committed_principal_count`, `committed_relation_count`, `last_seed_commit_id`, `program_activation_seed`, `recovered_publication`
- **bootstrap::managed_source_initialization**: 2 methods, 4 owned-field accesses; fields `pending_source_initialization`, `source_initializer`
- **bootstrap**: 3 methods, 3 owned-field accesses; fields `entity_keys`, `resource_support`
- **bootstrap::seed_batches**: 1 methods, 3 owned-field accesses; fields `entity_rows`, `relation_rows`, `rows`
- **bootstrap::publication_target**: 2 methods, 2 owned-field accesses; fields `runtime_authority`
- **typed_bootstrap**: 1 methods, 1 owned-field accesses; fields `pending_entity_keys`

## Cross-subsystem calls (would become narrow traits)

- bootstrap::advancement_publication -> bootstrap::managed_source_initialization: `take_source_initialization` (1)
- bootstrap::advancement_publication -> bootstrap::publication_target: `validate_publication_target` (1)
- bootstrap::seed_batches -> bootstrap::advancement_publication: `commit_seed_batch_once` (1)
- bootstrap::seed_batches -> bootstrap::managed_source_initialization: `take_source_initialization` (1)

## Fields used outside their subsystem

- bootstrap uses typed_bootstrap's `pending_entity_keys` (2 methods)
- bootstrap::advancement_publication uses bootstrap::seed_batches's `entity_rows` (2 methods)
- bootstrap::advancement_publication uses bootstrap::seed_batches's `relation_rows` (2 methods)
- bootstrap::advancement_publication uses bootstrap::seed_batches's `rows` (2 methods)
- bootstrap uses bootstrap::seed_batches's `entity_rows` (1 methods)
- bootstrap uses bootstrap::seed_batches's `rows` (1 methods)
- bootstrap::advancement_publication uses bootstrap's `resource_support` (1 methods)
- bootstrap::advancement_publication uses typed_bootstrap's `pending_entity_keys` (1 methods)
- bootstrap::managed_source_initialization uses bootstrap::advancement_publication's `recovered_publication` (1 methods)
- typed_bootstrap uses bootstrap::seed_batches's `relation_rows` (1 methods)
