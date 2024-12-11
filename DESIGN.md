# Design

We are heavily inspired by [SeaORM](https://github.com/SeaQL/sea-orm), [DoctrineORM](https://github.com/doctrine/orm), and official [SurrealDB SDK](https://github.com/surrealdb/surrealdb/tree/main/crates/sdk).

1. Intuitive and ergonomic

API should state the intention clearly. Provide syntax sugar for common things.

2. Fast(er) compilation

Balance between compile-time checking and compilation speed.

3. Avoid 'symbol soup'

Avoid macros with DSL, use derive macros where appropriate. Be friendly with IDE tools.
