use super::*;
use rxqlite_client::RXQLiteClientBuilder;

impl RXQLiteConnection {
    pub async fn establish(
        options: &RXQLiteConnectOptions,
    ) -> Result<Self, sqlx_core::error::Error> {
        let builder = RXQLiteClientBuilder::new(
          options.node_id,
          format!("{}:{}",options.node_host,options.node_port),
          )
          .tls_config(options.tls_config.clone());

        Ok(Self { inner: builder.build() })
    }
}
