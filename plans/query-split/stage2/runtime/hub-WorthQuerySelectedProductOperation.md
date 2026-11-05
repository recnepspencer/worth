# Hub map: `WorthQuerySelectedProductOperation`

- impl blocks: 56; methods: 132
- impl blocks by module: application_attempt 7, application_query 7, application_output_demand 6, product_operation::correction_source 6, application_discovery 5, entity_resolution 5, product_operation::program_adoption 4, product_operation::query 4, resolution 3, product_operation::context 2, conditional_operation 1, product_operation::branches 1, product_operation::conditional_definition 1, product_operation::generated_materialization 1, product_operation::operation 1, product_operation::relational_change_delivery 1, product_operation::shared_query_basis 1

## Fields, by the modules whose methods touch them

| field | owner | methods | modules (methods touching) |
|---|---|---|---|
| `product` | product_operation::context | 4 | product_operation::context 4 |
| `application` | product_operation::context | 3 | product_operation::context 3 |
| `application_basis` | product_operation::context | 3 | product_operation::context 3 |
| `producer` | product_operation::context | 3 | product_operation::context 3 |
| `selection_origin` | product_operation::context | 2 | product_operation::context 2 |
| `application.on_product()` | shared | 1 | product_operation::context 1 |
| `selected_program` | shared | 1 | product_operation::context 1 |

## Candidate subsystems (heaviest first)

- **product_operation::context**: 9 methods, 15 owned-field accesses; fields `application`, `application_basis`, `producer`, `product`, `selection_origin`

## Cross-subsystem calls (would become narrow traits)


## Fields used outside their subsystem

