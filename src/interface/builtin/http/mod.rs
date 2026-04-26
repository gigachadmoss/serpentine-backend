use std::{net::TcpListener, os::unix::net::UnixListener, path::PathBuf};

use actix_web::{App, HttpServer, dev::ServerHandle, get, web};
use serde::{Deserialize, Serialize};

use crate::interface::{
    InterfaceProvider, InterfaceProviderConfig, builtin::http::feature::FeatureSupport,
};

mod feature;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("TCP bind error: {0}")]
    TcpBind(std::io::Error),
    #[error("Unix socket bind error: {0}")]
    UnixBind(std::io::Error),
    #[error("HTTP server error: {0}")]
    HttpServer(std::io::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct HttpInterfaceProviderConfig {
    pub listeners: Vec<Listener>,
    /// Whether to shutdown entire backend when HTTP server exits.
    pub fails: bool,
}

impl InterfaceProviderConfig for HttpInterfaceProviderConfig {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Listener {
    /// (address, port)
    Tcp {
        addr: String,
        port: u16,
        fails: bool,
    },
    /// Unix socket path
    Unix { path: PathBuf, fails: bool },
}

#[derive(Clone)]
struct AppState {
    feature_support: FeatureSupport,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackendInformationResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub features: Vec<String>,
}

impl BackendInformationResponse {
    pub async fn get(feature_support: FeatureSupport) -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            features: feature_support.get().await,
        }
    }
}

/// No feature ID associated with this endpoint.
#[get("/api/info")]
pub async fn backend_information(
    state: web::Data<AppState>,
) -> web::Json<BackendInformationResponse> {
    web::Json(BackendInformationResponse::get(state.feature_support.clone()).await)
}

pub struct HttpInterfaceProvider {
    config: HttpInterfaceProviderConfig,
    handle: ServerHandle,
    death_tx: tokio::sync::broadcast::Sender<Result<(), ()>>,
}

impl InterfaceProvider for HttpInterfaceProvider {
    type Error = Error;
    type Config = HttpInterfaceProviderConfig;

    fn init(
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized,
    {
        async move {
            tracing::info!("Initializing HTTP server");

            let (handle, death_tx) = init(&config).await?;

            Ok(Self {
                config,
                handle: handle,
                death_tx,
            })
        }
    }
    async fn shutdown(&mut self) {
        tracing::info!("Shutting down HTTP server");

        self.handle.stop(true).await;
    }
    async fn wait(&mut self) {
        let mut rx = self.death_tx.subscribe();

        match rx.recv().await {
            Ok(_) => {
                tracing::info!("HTTP server exited");
            }
            Err(e) => {
                tracing::error!("Failed to receive HTTP server death signal: {}", e);
            }
        };
    }
    fn get_fails(&self) -> bool {
        self.config.fails
    }
}

// Initializes HTTP server
async fn init(
    config: &HttpInterfaceProviderConfig,
) -> Result<(ServerHandle, tokio::sync::broadcast::Sender<Result<(), ()>>), Error> {
    let listeners = config.listeners.clone();

    let feature_support = FeatureSupport::new();

    feature_support.register("test".to_string()).await;

    let state = AppState { feature_support };

    let mut http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(backend_information)
    });

    // Count the number of registered listeners
    let mut registered: usize = 0;

    for listener in listeners {
        match listener {
            Listener::Tcp { addr, port, fails } => {
                tracing::info!("Setting up TCP listener on {}:{}", addr, port);

                let l = match TcpListener::bind(format!("{}:{}", addr, port))
                    .map_err(Error::TcpBind)
                {
                    Ok(l) => l,
                    Err(e) => {
                        tracing::error!("Failed to bind TCP listener on {}:{} ({})", addr, port, e);

                        if fails {
                            return Err(e);
                        }

                        continue;
                    }
                };

                match http_server.listen(l).map_err(Error::HttpServer) {
                    Ok(server) => {
                        http_server = server;

                        registered += 1;
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to listen over TCP on with {}:{} ({})",
                            addr,
                            port,
                            e
                        );

                        return Err(e);
                    }
                };
            }
            Listener::Unix { path, fails } => {
                tracing::info!("Setting up Unix socket listener on {:?}", path);

                let l = match UnixListener::bind(&path).map_err(Error::UnixBind) {
                    Ok(l) => l,
                    Err(e) => {
                        tracing::error!("Failed to bind Unix socket listener on {:?}", path);

                        if fails {
                            return Err(e);
                        }

                        continue;
                    }
                };

                match http_server.listen_uds(l).map_err(Error::HttpServer) {
                    Ok(server) => {
                        http_server = server;

                        registered += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to listen on Unix socket listener on {:?}", path);
                        return Err(e);
                    }
                };
            }
        }
    }

    if registered == 0 {
        return Err(Error::HttpServer(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No listeners registered",
        )));
    }

    let server = http_server.run();

    // Makes it easier to kill later
    let handle = server.handle();

    // So we know when it dies
    let death_tx = tokio::sync::broadcast::channel::<Result<(), ()>>(1).0;

    let inner_death_tx = death_tx.clone();

    // Spawn server onto runtime background threads
    let _ = tokio::spawn(async move {
        let r = server.await.map_err(Error::HttpServer);

        match inner_death_tx.send(r.map_err(|_| ())) {
            Ok(_) => {}
            Err(r) => match r.0 {
                Ok(_) => {
                    tracing::error!(
                        "Failed to send HTTP server death signal, but HTTP server appears to have exited without error"
                    );
                }
                Err(_) => {
                    tracing::error!(
                        "Failed to send HTTP server death signal, and got HTTP server error"
                    );
                }
            },
        }
    });

    tracing::info!("HTTP server started successfully");

    Ok((handle, death_tx))
}
