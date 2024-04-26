use super::*;
use sqlx::types::chrono::{Utc};


#[test]
fn invalid_queries() {
    let rt = Runtime::new().unwrap();
    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let /*mut*/ tm = TestManager::new("invalid_queries",3,None);
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        let pool = RXQLitePoolOptions::new()
            //.max_connections(5)
            .connect(&format!(
                "rxqlite://{}",
                tm.instances.get(&1).unwrap().http_addr
            ))
            .await
            .unwrap();
         let res = sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_rxqlite_test_user_and_date_ (
        id abcdef PRIMARY KEY
    )",
        )
        .execute(&pool)
        .await;
         
        assert!(res.is_ok());
         let res = sqlx::query(
            "DROP TABLE _sqlx_rxqlite_test_user_and_date_",
        )
        .execute(&pool)
        .await;
        assert!(res.is_ok());
        
        let res = sqlx::query(
            "CREATE MENSA IF NOT EXISTS _sqlx_rxqlite_test_user_and_date_ (
        id abcdef PRIMARY KEY
    )",
        )
        .execute(&pool)
        .await;
         
        assert!(res.is_err());
        
        let res = sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_rxqlite_test_user_and_date_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        birth_date DATETIME NOT NULL,
    )",
        )
        .execute(&pool)
        .await;
        assert!(res.is_err());
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_rxqlite_test_user_and_date_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        birth_date DATETIME NOT NULL
    )",
        )
        .execute(&pool)
        .await.unwrap();
        let birth_date = Utc::now();
        let name = "Ha";
        let res = sqlx::query(
            "INSERT INTO _sqlx_rxqlite_test_user_and_date_ (name,birth_date) VALUES (?, ?);",
        )
        .bind(&birth_date)
        .execute(&pool)
        .await;
        assert!(res.is_err());
        sqlx::query(
            "INSERT INTO _sqlx_rxqlite_test_user_and_date_ (name,birth_date) VALUES (?, ?);",
        )
        .bind(&name)
        .bind(&birth_date)
        .execute(&pool)
        .await.unwrap();
        let rows = sqlx::query("SELECT FROM _sqlx_rxqlite_test_user_and_date_ WHERE name = ?")
            .bind(name)
            .fetch_all(&pool)
            .await;
        assert!(rows.is_err());
        let rows = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_and_date_ WHERE name = ?")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 0);
        
        let rows = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_and_date_ WHERE name = ?")
            .bind(name)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    });
}
