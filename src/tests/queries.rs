use super::*;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Row;

use futures::future::join_all;
use rand::Rng;

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

#[test]
fn many_connections() {
    let rt = Runtime::new().unwrap();
    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let /*mut*/ tm = TestManager::new("many_connections",3,None);
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        const CONNECTIONS_COUNT:usize=100;
        let mut futs = vec![];
        let mut rng = rand::thread_rng();
        for i in 0..CONNECTIONS_COUNT {
          let node_id = rng.gen_range(1..tm.instances.len()+1);
          let http_addr = tm.instances.get(&(node_id as u64)).unwrap().http_addr.clone();
          futs.push(async move{
            let pool = RXQLitePoolOptions::new()
                //.max_connections(5)
                .connect(&format!(
                    "rxqlite://{}",
                    http_addr
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
            let name = format!("Ha-{}",i);
            sqlx::query(
                "INSERT INTO _sqlx_rxqlite_test_user_and_date_ (name,birth_date) VALUES (?, ?);",
            )
            .bind(&name)
            .bind(&birth_date)
            .execute(&pool)
            .await
            .unwrap();

            let rows = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_and_date_ WHERE name = ?")
                .bind(&name)
                .fetch_all(&pool)
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            let row = &rows[0];
            let fetched_name: String = row.get(1);
            assert_eq!(&fetched_name, &name);
            let fetched_birth_date: DateTime<Utc> = row.get(2);
            assert_eq!(fetched_birth_date, birth_date);
          });
        }
        join_all(futs).await;
    });
}
