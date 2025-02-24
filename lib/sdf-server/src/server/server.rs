use std::{io, net::SocketAddr, path::Path, path::PathBuf, sync::Arc};

use crate::server::config::CycloneKeyPair;
use axum::routing::IntoMakeService;
use axum::Router;
use dal::tasks::{StatusReceiver, StatusReceiverError};
use dal::JwtPublicSigningKey;
use dal::{
    cyclone_key_pair::CycloneKeyPairError, job::processor::JobQueueProcessor,
    tasks::ResourceScheduler, ServicesContext,
};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolConfig, PgPoolError};
use si_posthog::{PosthogClient, PosthogConfig};
use si_std::SensitiveString;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal,
    sync::{broadcast, mpsc, oneshot},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use veritech_client::{Client as VeritechClient, EncryptionKey, EncryptionKeyError};

use super::state::AppState;
use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("cyclone public key already set")]
    CyclonePublicKeyAlreadySet,
    #[error("cyclone public key error: {0}")]
    CyclonePublicKeyErr(#[from] CycloneKeyPairError),
    #[error(transparent)]
    DalInitialization(#[from] dal::InitializationError),
    #[error("error when loading encryption key: {0}")]
    EncryptionKey(#[from] EncryptionKeyError),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("error initializing the server")]
    Init,
    #[error("jwt secret key error")]
    JwtSecretKey(#[from] dal::jwt_key::JwtKeyError),
    #[error(transparent)]
    Model(#[from] dal::ModelError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error(transparent)]
    Posthog(#[from] si_posthog::PosthogError),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    StatusReceiver(#[from] StatusReceiverError),
    #[error(transparent)]
    Uds(#[from] UdsIncomingStreamError),
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
}

impl From<PgPoolError> for ServerError {
    fn from(value: PgPoolError) -> Self {
        Self::PgPool(Box::new(value))
    }
}

pub type Result<T, E = ServerError> = std::result::Result<T, E>;

pub struct Server<I, S> {
    config: Config,
    inner: axum::Server<I, IntoMakeService<Router>>,
    socket: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server<(), ()> {
    #[allow(clippy::too_many_arguments)]
    pub fn http(
        config: Config,
        pg_pool: PgPool,
        nats: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: EncryptionKey,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        pkgs_path: PathBuf,
        module_index_url: String,
    ) -> Result<(Server<AddrIncoming, SocketAddr>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let services_context = ServicesContext::new(
                    pg_pool,
                    nats,
                    job_processor,
                    veritech,
                    Arc::new(encryption_key),
                    Some(pkgs_path),
                    Some(module_index_url),
                );

                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    config.signup_secret().clone(),
                    posthog_client,
                )?;

                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                let inner = axum::Server::bind(socket_addr).serve(service.into_make_service());
                let socket = inner.local_addr();

                Ok((
                    Server {
                        config,
                        inner,
                        socket,
                        shutdown_rx,
                    },
                    shutdown_broadcast_rx,
                ))
            }
            wrong @ IncomingStream::UnixDomainSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn uds(
        config: Config,
        pg_pool: PgPool,
        nats: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: EncryptionKey,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        pkgs_path: PathBuf,
        module_index_url: String,
    ) -> Result<(Server<UdsIncomingStream, PathBuf>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let services_context = ServicesContext::new(
                    pg_pool,
                    nats,
                    job_processor,
                    veritech,
                    Arc::new(encryption_key),
                    Some(pkgs_path),
                    Some(module_index_url),
                );

                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    config.signup_secret().clone(),
                    posthog_client,
                )?;

                info!("binding to Unix domain socket; path={}", path.display());
                let inner = axum::Server::builder(UdsIncomingStream::create(path).await?)
                    .serve(service.into_make_service());
                let socket = path.clone();

                Ok((
                    Server {
                        config,
                        inner,
                        socket,
                        shutdown_rx,
                    },
                    shutdown_broadcast_rx,
                ))
            }
            wrong @ IncomingStream::HTTPSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    pub fn init() -> Result<()> {
        Ok(dal::init()?)
    }

    pub async fn start_posthog(config: &PosthogConfig) -> Result<PosthogClient> {
        let (posthog_client, posthog_sender) = si_posthog::from_config(config)?;

        drop(tokio::spawn(posthog_sender.run()));

        Ok(posthog_client)
    }

    #[instrument(name = "sdf.init.generate_cyclone_key_pair", skip_all)]
    pub async fn generate_cyclone_key_pair(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<()> {
        CycloneKeyPair::create(secret_key_path, public_key_path)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "sdf.init.load_jwt_public_signing_key", skip_all)]
    pub async fn load_jwt_public_signing_key(
        path: impl AsRef<Path>,
    ) -> Result<JwtPublicSigningKey> {
        Ok(JwtPublicSigningKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.load_encryption_key", skip_all)]
    pub async fn load_encryption_key(path: impl AsRef<Path>) -> Result<EncryptionKey> {
        Ok(EncryptionKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.migrate_database", skip_all)]
    pub async fn migrate_database(
        pg: &PgPool,
        nats: &NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: &EncryptionKey,
        pkgs_path: PathBuf,
        module_index_url: String,
    ) -> Result<()> {
        dal::migrate_all_with_progress(
            pg,
            nats,
            job_processor,
            veritech,
            encryption_key,
            pkgs_path,
            module_index_url,
        )
        .await?;
        Ok(())
    }

    /// Start the basic resource refresh scheduler
    pub async fn start_resource_refresh_scheduler(
        pg: PgPool,
        nats: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: EncryptionKey,
        shutdown_broadcast_rx: broadcast::Receiver<()>,
    ) {
        let services_context = ServicesContext::new(
            pg,
            nats,
            job_processor,
            veritech,
            Arc::new(encryption_key),
            None,
            None,
        );
        ResourceScheduler::new(services_context).start(shutdown_broadcast_rx);
    }

    pub async fn start_status_updater(
        pg: PgPool,
        nats: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: EncryptionKey,
        shutdown_broadcast_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        let services_context = ServicesContext::new(
            pg,
            nats,
            job_processor,
            veritech,
            Arc::new(encryption_key),
            None,
            None,
        );
        StatusReceiver::new(services_context)
            .await?
            .start(shutdown_broadcast_rx);
        Ok(())
    }

    #[instrument(name = "sdf.init.create_pg_pool", skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "sdf.init.connect_to_nats", skip_all)]
    pub async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    pub fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }
}

impl<I, IO, IE, S> Server<I, S>
where
    I: Accept<Conn = IO, Error = IE>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn run(self) -> Result<()> {
        let shutdown_rx = self.shutdown_rx;

        self.inner
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .map_err(Into::into)
    }

    /// Gets a reference to the server's config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Gets a reference to the server's locally bound socket.
    pub fn local_socket(&self) -> &S {
        &self.socket
    }
}

pub fn build_service_for_tests(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        signup_secret,
        posthog_client,
        true,
    )
}

pub fn build_service(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        signup_secret,
        posthog_client,
        false,
    )
}

fn build_service_inner(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
    for_tests: bool,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let state = AppState::new(
        services_context,
        signup_secret,
        jwt_public_signing_key,
        posthog_client,
        shutdown_broadcast_tx.clone(),
        shutdown_tx,
        for_tests,
    );

    let routes = routes(state)
        // TODO(fnichol): customize http tracing further, using:
        // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let graceful_shutdown_rx = prepare_graceful_shutdown(shutdown_rx, shutdown_broadcast_tx)?;

    Ok((routes, graceful_shutdown_rx, shutdown_broadcast_rx))
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_watcher =
        signal::unix::signal(signal::unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(
            tx: oneshot::Sender<()>,
            shutdown_broadcast_tx: broadcast::Sender<()>,
        ) {
            // Send graceful shutdown to axum server which stops it from accepting requests
            if tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
            // Send shutdown to all long running sessions (notably, WebSocket sessions), so they
            // can cleanly terminate
            if shutdown_broadcast_tx.send(()).is_err() {
                error!("all broadcast shutdown receivers have already been dropped");
            }
        }

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("received SIGINT signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            _ = sigterm_watcher.recv() => {
                info!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            source = shutdown_rx.recv() => {
                info!(
                    "received internal shutdown, performing graceful shutdown; source={:?}",
                    source,
                );
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            else => {
                // All other arms are closed, nothing left to do but return
                trace!("returning from graceful shutdown with all select arms closed");
            }
        };
    });

    Ok(graceful_shutdown_rx)
}

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}
