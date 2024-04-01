use super::*;
use rxqlite::tests::*;
use tokio::runtime::Runtime;
use rxqlite_tests_common::TestTlsConfig;
use rxqlite_tests_common::TestClusterManager;
use rxqlite_lite_common::{RaftMetrics,LogId};
use std::path::PathBuf;
use std::env;
use rxqlite_lite_common::NodeId;
use sqlx::Pool;
use futures_util::future::join_all;

#[cfg(target_os = "windows")]
const EXE_SUFFIX: &str = ".exe";

#[cfg(not(target_os = "windows"))]
const EXE_SUFFIX: &str = "";



pub fn get_cluster_manager(
    test_name: &str,
    instance_count: usize,
    tls_config: Option<TestTlsConfig>,
) -> anyhow::Result<TestClusterManager> {
    let executable_path = if let Ok(rxqlited_dir) = std::env::var("RXQLITED_DIR") {
        let executable_path = PathBuf::from(rxqlited_dir).join(format!("rxqlited{}", EXE_SUFFIX));
        println!("using rxqlited: {}", executable_path.display());
        executable_path
    } else {
        let cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let executable_path = cargo_path
            .join("target")
            .join("release")
            .join(format!("rxqlited{}", EXE_SUFFIX));
        println!("using rxqlited: {}", executable_path.display());
        executable_path
    };
    assert!(executable_path.is_file());
    let temp_dir = env::temp_dir();
    let working_directory = temp_dir.join(test_name);
    TestClusterManager::new(
        instance_count,
        &working_directory,
        &executable_path,
        "127.0.0.1",
        tls_config,
    )
}

pub struct TestManagerWithPool {
    pub tcm: TestClusterManager,
    pub pools: HashMap<NodeId, Pool<RXQLite> >,
}

impl std::ops::Deref for TestManagerWithPool {
    type Target = TestClusterManager;
    fn deref(&self) -> &Self::Target {
        &self.tcm
    }
}

impl std::ops::DerefMut for TestManagerWithPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tcm
    }
}

impl TestManagerWithPool {
    pub async fn new(test_name: &str, instance_count: usize, tls_config: Option<TestTlsConfig>) -> anyhow::Result<Self> {
        let tcm = get_cluster_manager(test_name, instance_count, tls_config.clone()).unwrap();
        //let mut pools: HashMap<NodeId, Pool<RXQLite>> = Default::default();
        
        let pools = tcm
            .instances
            .iter()
            .map(|(node_id, instance)| {
              let tls_config=tls_config.clone();
              async move{
                let mut connect_options = RXQLiteConnectOptions::default();
                if let Some(tls_config)=tls_config {
                    connect_options = connect_options.use_ssl(true);
                    connect_options = connect_options.use_insecure_ssl(tls_config.accept_invalid_certificates);
                    
                  }
                  connect_options = connect_options.leader_id(*node_id);
                  let mut split= instance.http_addr.split(":");
                  
                  connect_options = connect_options.host(split.next().unwrap());
                  connect_options = connect_options.port(split.next().unwrap().parse::<u16>().unwrap());
                  
                let pool_options = RXQLitePoolOptions::new();
                  
                  //.max_connections(5)
                  (
                    *node_id,
                    pool_options.connect_with(connect_options)
                      .await
                  )
              }
            })
            .collect::<Vec<_>>();
        let pools_ = join_all(pools).await;
        let mut pools: HashMap<NodeId, Pool<RXQLite>> = Default::default();
        for (node_id,pool) in pools_.into_iter() {
          pools.insert(node_id,pool?);
        }
        Ok(Self { tcm, pools })
    }

    pub async fn get_metrics(&self, node_id: NodeId) -> anyhow::Result<RaftMetrics> {
        let pool = self.pools.get(&node_id).unwrap();
        let conn = pool.acquire().await?;
        let metrics = conn.inner.metrics().await?;
        Ok(metrics)
    }
    pub fn node_count(&self) -> usize {
        self.tcm.instances.len()
    }

    pub async fn wait_for_cluster_established(
        &self,
        node_id: NodeId,
        reattempts: usize,
    ) -> anyhow::Result<()> {
        let mut reattempts = reattempts + 1; // wait max for cluster to establish

        loop {
            if let Ok(metrics) = self.get_metrics(node_id).await {
              if metrics.current_leader.is_some() {
                let voter_ids = metrics.membership_config.voter_ids();
                if voter_ids.count() == self.node_count() {
                  return Ok(());
                }
              }
            }
            reattempts -= 1;
            if reattempts == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Err(anyhow::anyhow!("wait_for_cluster_established timeout"))
    }
    #[allow(dead_code)]
    pub async fn wait_for_last_applied_log(
        &self,
        log_id: LogId,
        reattempts: usize,
    ) -> anyhow::Result<HashMap<NodeId, RaftMetrics>> {
        let mut reattempts = reattempts + 1;
        let mut node_metrics: HashMap<NodeId, RaftMetrics> = Default::default();

        loop {
            let mut futs = vec![];
            for (node_id, pool) in self.pools.iter() {
                if node_metrics.contains_key(node_id) {
                    continue;
                }
                let conn = pool.acquire().await?;
                futs.push(async move {
                  conn.inner.metrics().await
                });
            }
            if futs.len() == 0 {
                return Ok(node_metrics);
            }
            let metrics = join_all(futs).await;
            for metrics in metrics {
                if let Ok(metrics) = metrics {
                    if let Some(last_applied) = metrics.last_applied {
                        if last_applied >= log_id {
                            node_metrics.insert(metrics.id, metrics);
                        }
                    }
                }
            }
            if node_metrics.len() == self.pools.len() {
                return Ok(node_metrics);
            }
            reattempts -= 1;
            if reattempts == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Err(anyhow::anyhow!("wait_for_last_applied_log timeout"))
    }
}

pub mod consts;
mod queries;
mod invalid_queries;
mod pool_notifications;
mod pool_notifications2;
