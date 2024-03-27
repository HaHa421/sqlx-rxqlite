use super::*;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Row;

#[test]
fn simple_query() {
    let rt = Runtime::new().unwrap();
    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let /*mut*/ tm = TestManager::new("simple_query",3,None);
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        let pool = RXQLitePoolOptions::new()
            //.max_connections(5)
            .connect(&format!(
                "rxqlite://{}",
                tm.instances.get(&1).unwrap().http_addr
            ))
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_rxqlite_test_user_and_date_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        birth_date DATETIME NOT NULL
    )",
        )
        .execute(&pool)
        .await
        .unwrap();
        let birth_date = Utc::now();
        let name = "Ha";
        sqlx::query(
            "INSERT INTO _sqlx_rxqlite_test_user_and_date_ (name,birth_date) VALUES (?, ?);",
        )
        .bind(name)
        .bind(&birth_date)
        .execute(&pool)
        .await
        .unwrap();

        let rows = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_and_date_ WHERE name = ?")
            .bind(name)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        let fetched_name: String = row.get(1);
        assert_eq!(&fetched_name, name);
        let fetched_birth_date: DateTime<Utc> = row.get(2);
        assert_eq!(fetched_birth_date, birth_date);
    });
}
