use rxqlite_common::RSQliteClientTlsConfig;

#[derive(Debug, Clone , Default)]
pub struct RXQLiteConnectOptions {
    //pub(crate) inner: rxqlite::ConnectOptions,
    //pub(crate) inner: rxqlite_client::RXQLiteClientBuilder,
    
    pub(crate) node_id: u64,
    pub(crate) node_host: String,
    pub(crate) node_port: u16,
    pub(crate) tls_config: Option<RSQliteClientTlsConfig>,
}

impl RXQLiteConnectOptions {
    pub fn node_id(mut self, node_id: u64) -> Self {
        self.node_id = node_id;
        self
    }
    pub fn host(mut self, host: &str) -> Self {
        self.node_host = host.to_owned();
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.node_port = port;
        self
    }

    /*
    pub fn user(mut self, user: Option<String>) -> Self {
      self.inner.user = user;
      self
    }
    pub fn password(mut self, pwd: Option<String>) -> Self {
      self.inner.pass = pwd;
      self
    }
    */
    pub fn use_ssl(mut self, use_ssl: bool) -> Self {
        if use_ssl {
            //rxqlite::Scheme::HTTPS
            self.tls_config = Some(Default::default());
            self.tls_config
                .as_mut()
                .unwrap()
                .accept_invalid_certificates = false;
        } else {
            self.tls_config = None;
        }
        self
    }
    pub fn use_insecure_ssl(mut self, use_insecure_ssl: bool) -> Self {
        if self.tls_config.is_none() {
            self.tls_config = Some(Default::default());
        }
        if use_insecure_ssl {
            //self.scheme = rxqlite::Scheme::HTTPS;
            self.tls_config
                .as_mut()
                .unwrap()
                .accept_invalid_certificates = true;
        } else {
            self.tls_config
                .as_mut()
                .unwrap()
                .accept_invalid_certificates = false;
            //self.scheme = rxqlite::Scheme::HTTP;
        }
        self
    }
    pub fn add_cert_path(mut self, cert_path: String) -> Self {
        if self.tls_config.is_none() {
            self.tls_config = Some(Default::default());
        }
        self.tls_config
            .as_mut()
            .unwrap()
            .cert_paths
            .push(cert_path);
        self
    }
    pub fn tls_config(mut self, tls_config: Option<RSQliteClientTlsConfig>) -> Self {
        self.tls_config = tls_config;
        self
    }
}

pub mod connect;
mod parse;
