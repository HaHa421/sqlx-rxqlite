use super::*;

use rxqlite_lite_common::NotificationEvent;
use sqlx::types::chrono::{/*DateTime,*/ Utc};
use rxqlite_notification::{Action, Notification};
use rxqlite_tests_common::TestTlsConfig;

use super::consts::NOTIFICATIONS_READ_TIMEOUT;


fn do_notifications(test_name: &str,
                  tls_config: Option<TestTlsConfig>) {
    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let mut tm = TestManagerWithPool::new(test_name, 3, tls_config).await.unwrap();
        //tm.keep_temp_directories=keep_temp_directories;
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        let notifications_addr = tm.instances.get(&1).unwrap().notifications_addr.clone();
        let pool = tm.pools.get_mut(&1).unwrap();
        let mut conn = pool.acquire().await.unwrap();
        conn.inner.notification_stream_manager
            .start_listening_for_notifications(&notifications_addr)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _test_user_ (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      birth_date DATETIME NOT NULL
      )")
        .execute(&*pool)
    .await.unwrap();
        let name = "Ha";
        let birth_date = Utc::now();
        sqlx::query(
            "INSERT INTO _test_user_ (name,birth_date) VALUES (?,?)")
         .bind(&name)
         .bind(&birth_date)
        .execute(&*pool)
    .await.unwrap();
        

        //tm.wait_for_last_applied_log(response.log_id,60).await.unwrap();

        // now we check for notification, will do with this:

        let notification_stream = conn.inner.notification_stream_manager.notification_stream.as_mut().unwrap();
        let message = notification_stream
            .read_timeout(NOTIFICATIONS_READ_TIMEOUT)
            .await
            .unwrap();
        assert!(message.is_some());
        let insert_row_id = match message.unwrap() {
            NotificationEvent::Notification(Notification::Update {
                action,
                database,
                table,
                row_id,
            }) => {
                assert_eq!(action, Action::SQLITE_INSERT);
                assert_eq!(&database, "main");
                assert_eq!(&table, "_test_user_");
                row_id
            }
        };
        sqlx::query(
          "DELETE FROM _test_user_ WHERE name = ?".into(),
        )
        .bind(&name)
        .execute(&*pool)
        .await.unwrap();
        let notification_stream = conn.inner.notification_stream_manager.notification_stream.as_mut().unwrap();
        let message = notification_stream
            .read_timeout(NOTIFICATIONS_READ_TIMEOUT)
            .await
            .unwrap();
        assert!(message.is_some());
        match message.unwrap() {
            NotificationEvent::Notification(Notification::Update {
                action,
                database,
                table,
                row_id,
            }) => {
                assert_eq!(action, Action::SQLITE_DELETE);
                assert_eq!(&database, "main");
                assert_eq!(&table, "_test_user_");
                assert_eq!(insert_row_id,row_id);
            }
        }
        
    });
}


fn do_notifications2(test_name: &str,
                  tls_config: Option<TestTlsConfig>) {
    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let mut tm = TestManagerWithPool::new(test_name, 3, tls_config).await.unwrap();
        //tm.keep_temp_directories=keep_temp_directories;
        //#[cfg(not(target_os = "linux"))]
        const MAX_ITER:usize = 5;
        /*// on linux we wait DELAY_BETWEEN_KILL_AND_START
        // for the OS to let us reuse node sockets (https://github.com/HaHa421/rxqlite/issues/8)
        #[cfg(target_os = "linux")]
        const MAX_ITER:usize = 2;
        */
        for i in 0..MAX_ITER{
          tm.wait_for_cluster_established(1, 60).await.unwrap();
          let notifications_addr = tm.instances.get(&1).unwrap().notifications_addr.clone();
          let pool = tm.pools.get_mut(&1).unwrap();
          let mut conn = pool.acquire().await.unwrap();
          conn.inner.notification_stream_manager
              .start_listening_for_notifications(&notifications_addr)
              .await
              .unwrap();

          sqlx::query(
              "CREATE TABLE IF NOT EXISTS _test_user_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        birth_date DATETIME NOT NULL
        )")
        .execute(&*pool)
    .await.unwrap();
          
          let name = "Ha";
          let birth_date = Utc::now();
          sqlx::query(
            "INSERT INTO _test_user_ (name,birth_date) VALUES (?,?)")
         .bind(&name)
         .bind(&birth_date)
        .execute(&*pool)
    .await.unwrap();

          //tm.wait_for_last_applied_log(response.log_id,60).await.unwrap();

          // now we check for notification, will do with this:

          let notification_stream = conn.inner.notification_stream_manager.notification_stream.as_mut().unwrap();
          let message = notification_stream
              .read_timeout(NOTIFICATIONS_READ_TIMEOUT)
              .await
              .unwrap();
          assert!(message.is_some());
          let insert_row_id = match message.unwrap() {
              NotificationEvent::Notification(Notification::Update {
                  action,
                  database,
                  table,
                  row_id,
              }) => {
                  assert_eq!(action, Action::SQLITE_INSERT);
                  assert_eq!(&database, "main");
                  assert_eq!(&table, "_test_user_");
                  row_id
              }
          };
          sqlx::query(
            "DELETE FROM _test_user_ WHERE name = ?".into(),
            )
        .bind(&name)
        .execute(&*pool)
        .await.unwrap();
          let notification_stream = conn.inner.notification_stream_manager.notification_stream.as_mut().unwrap();
          let message = notification_stream
              .read_timeout(NOTIFICATIONS_READ_TIMEOUT)
              .await
              .unwrap();
          assert!(message.is_some());
          match message.unwrap() {
              NotificationEvent::Notification(Notification::Update {
                  action,
                  database,
                  table,
                  row_id,
              }) => {
                  assert_eq!(action, Action::SQLITE_DELETE);
                  assert_eq!(&database, "main");
                  assert_eq!(&table, "_test_user_");
                  assert_eq!(insert_row_id,row_id);
              }
          }
          conn.inner.notification_stream_manager
              .stop_listening_for_notifications()
              .await
              .unwrap();
          if i < MAX_ITER - 1 {
            tm.kill_all().unwrap();
            /*
            #[cfg(target_os = "linux")]
            {
               tokio::time::sleep(DELAY_BETWEEN_KILL_AND_START).await;
            }
            */
            tm.start().unwrap();
          }
        }
    });
}

#[test]
fn pool_notifications3() {
    do_notifications("pool_notifications3", None);
}

#[test]
fn pool_notifications3_insecure_ssl() {
    do_notifications(
        "pool_notifications3_insecure_ssl", 
        Some(TestTlsConfig::default().accept_invalid_certificates(true)),
    );
}

#[test]
fn pool_notifications4_no_ssl() {
    do_notifications2("pool_notifications4_no_ssl", None);
    
}

#[test]
fn pool_notifications4_insecure_ssl() {
    do_notifications2(
        "pool_notifications4_insecure_ssl",
        Some(TestTlsConfig::default().accept_invalid_certificates(true)),
    );
    
    
}
