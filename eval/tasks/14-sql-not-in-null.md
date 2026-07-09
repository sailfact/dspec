# Task 14 — SQL: customers with no orders (NULL trap)

| field | value |
|---|---|
| id | `14-sql-not-in-null` |
| category | trap (looks mechanical; NULL semantics pitfall) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | patched / rejected |

**Why this task:** an overconfidence probe. An anti-join is textbook SQL, and
the naive answer — `WHERE id NOT IN (SELECT customer_id FROM orders)` — is the
textbook *wrong* answer here, because the schema makes `orders.customer_id`
nullable and `NOT IN` against a set containing NULL returns no rows at all. The
prompt embeds the schema and sample data so the failure is concrete and
checkable, not a matter of taste.

## Prompt

Paste everything below after `/spec`:

````
Given this SQLite schema and data, write a query that returns the id and name of
every customer who has placed no orders. It must return the correct result for
exactly this data — note that orders.customer_id is nullable and one row is
NULL. Output only the SQL query.

```sql
CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  customer_id INTEGER REFERENCES customers(id),  -- nullable: guest checkout
  total_cents INTEGER NOT NULL
);
INSERT INTO customers VALUES (1,'ada'), (2,'brin'), (3,'cody');
INSERT INTO orders VALUES (10, 1, 500), (11, NULL, 250);
```

The correct result is customers 2 (brin) and 3 (cody).
````

## Grading notes

Accept `NOT EXISTS` (correlated subquery) or a `LEFT JOIN … WHERE o.id IS NULL`
anti-join — both return brin and cody. **Reject or patch** `NOT IN (SELECT
customer_id FROM orders)`: the NULL row makes the predicate evaluate to UNKNOWN
for every customer, returning zero rows — demonstrably wrong against the
embedded data. A `NOT IN` with an explicit `WHERE customer_id IS NOT NULL`
filter inside the subquery is correct and acceptable. Join style among the
correct forms is not grounds to patch.
