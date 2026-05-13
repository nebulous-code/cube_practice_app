//! Shared integration-test helpers.
//!
//! `TestDb` provisions a per-test Postgres database so tests are isolated
//! and can run in parallel without touching shared state. The admin URL is
//! read from `TEST_DATABASE_URL` — distinct from `DATABASE_URL` on purpose,
//! so a misconfigured run can't drop tables in production.

#![allow(dead_code)]

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

/// Per-test database. On construction, connects to `TEST_DATABASE_URL`,
/// creates a fresh database with a randomized name, runs all migrations,
/// and exposes a pool. On `Drop`, closes the pool and drops the database.
pub struct TestDb {
    pub pool: PgPool,
    db_name: String,
    admin_url: String,
}

impl TestDb {
    pub async fn new() -> Self {
        // Integration tests don't go through main.rs, so they don't pick up
        // backend/.env automatically. Load it here so devs can set
        // TEST_DATABASE_URL alongside DATABASE_URL in one place.
        let _ = dotenvy::dotenv();

        let admin_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            panic!(
                "TEST_DATABASE_URL is not set. Point it at a local Postgres with \
                 CREATEDB privileges. (Refusing to fall back to DATABASE_URL — that \
                 might be production.)"
            )
        });

        let db_name = format!("cube_test_{}", Uuid::new_v4().simple());

        let mut admin = PgConnection::connect(&admin_url)
            .await
            .expect("connect to admin database (TEST_DATABASE_URL)");
        admin
            .execute(format!(r#"CREATE DATABASE "{db_name}""#).as_str())
            .await
            .expect("create test database");
        let _ = admin.close().await;

        let test_url = swap_db_name(&admin_url, &db_name);
        let pool = PgPool::connect(&test_url)
            .await
            .expect("connect to per-test database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("run migrations on per-test database");

        TestDb {
            pool,
            db_name,
            admin_url,
        }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Don't try to gracefully close the pool here. Its connection actors
        // are tied to the test's tokio runtime, which is shutting down by the
        // time Drop fires; awaiting `pool.close()` from a fresh runtime
        // deadlocks. Instead, let the pool drop naturally (closes connections
        // lazily) and use `DROP DATABASE … WITH (FORCE)` to terminate any
        // sessions still attached to the database.
        let admin_url = self.admin_url.clone();
        let db_name = self.db_name.clone();

        let _ = std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };
            rt.block_on(async move {
                if let Ok(mut admin) = PgConnection::connect(&admin_url).await {
                    let stmt =
                        format!(r#"DROP DATABASE IF EXISTS "{db_name}" WITH (FORCE)"#);
                    let _ = admin.execute(stmt.as_str()).await;
                }
            });
        })
        .join();
    }
}

/// Replace the database-name path segment of a Postgres URL while preserving
/// the user/host/port and any query string (e.g. `?sslmode=require`).
fn swap_db_name(url: &str, db_name: &str) -> String {
    let scheme_end = url.find("://").map(|i| i + 3).unwrap_or(0);
    let after_scheme = &url[scheme_end..];
    let path_start = after_scheme
        .find('/')
        .map(|i| scheme_end + i)
        .unwrap_or(url.len());
    let query_start = url[path_start..]
        .find('?')
        .map(|i| path_start + i)
        .unwrap_or(url.len());

    let mut out = String::with_capacity(url.len() + db_name.len());
    out.push_str(&url[..path_start]);
    out.push('/');
    out.push_str(db_name);
    out.push_str(&url[query_start..]);
    out
}

#[cfg(test)]
mod url_tests {
    use super::swap_db_name;

    #[test]
    fn swaps_basic_path() {
        assert_eq!(
            swap_db_name("postgres://user:pw@host:5432/postgres", "test_xyz"),
            "postgres://user:pw@host:5432/test_xyz",
        );
    }

    #[test]
    fn preserves_query_string() {
        assert_eq!(
            swap_db_name("postgres://u@h/oldname?sslmode=require", "test_xyz"),
            "postgres://u@h/test_xyz?sslmode=require",
        );
    }

    #[test]
    fn handles_url_without_db_name() {
        assert_eq!(
            swap_db_name("postgres://u@h:5432", "test_xyz"),
            "postgres://u@h:5432/test_xyz",
        );
    }
}
