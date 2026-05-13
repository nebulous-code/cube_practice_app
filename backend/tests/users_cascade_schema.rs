//! Schema-introspection guard. Every FK that points at `users.id` must
//! cascade on delete, otherwise `account_delete::delete_account` either
//! orphans rows or fails outright.
//!
//! See docs/milestones/07_delete_account.md §3 + §6.

mod common;

use common::TestDb;

#[tokio::test]
async fn every_fk_referencing_users_id_has_cascade_delete() {
    let db = TestDb::new().await;

    // Walks information_schema.referential_constraints to find every FK
    // whose referenced column is `users.id`. Joins through key_column_usage
    // so we can return both ends of each FK in the failure message.
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            tc.table_name      AS referencing_table,
            kcu.column_name    AS referencing_column,
            tc.constraint_name AS constraint_name,
            rc.delete_rule     AS delete_rule
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
         AND tc.table_schema = kcu.table_schema
        JOIN information_schema.referential_constraints rc
          ON tc.constraint_name = rc.constraint_name
         AND tc.table_schema = rc.constraint_schema
        JOIN information_schema.constraint_column_usage ccu
          ON rc.unique_constraint_name = ccu.constraint_name
         AND rc.unique_constraint_schema = ccu.constraint_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
          AND ccu.table_name = 'users'
          AND ccu.column_name = 'id'
          AND tc.table_schema = 'public'
        "#,
    )
    .fetch_all(&db.pool)
    .await
    .expect("introspect FK rules");

    assert!(
        !rows.is_empty(),
        "no FKs found referencing users.id — schema introspection broke",
    );

    let bad: Vec<_> = rows
        .iter()
        .filter(|(_, _, _, rule)| rule != "CASCADE")
        .collect();

    assert!(
        bad.is_empty(),
        "every FK referencing users.id must ON DELETE CASCADE; offenders: {:#?}",
        bad,
    );
}
