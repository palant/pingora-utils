// Copyright 2024 Wladimir Palant
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Startup Module for Pingora
//!
//! This crate helps configure and set up the Pingora server. It provides a [`StartupOpt`] data
//! structure with the relevant command line and [`StartupConf`] with the configuration file
//! options. Once these data structures are all set up, [`StartupConf::into_server`] method can be
//! used to get a Pingora server instance.
//!
//! ## General configuration
//!
//! The Startup Module currently exposes all of the
//! [Pingora configuration options](module_utils::pingora::ServerConf). In addition, it provides
//! a `listen` configuration option, a list of IP address/port combinations that the server should
//! listen on:
//!
//! ```yaml
//! listen:
//! - 127.0.0.1:8080
//! - "[::1]:8080"
//! ```
//!
//! On many Unix and Linux systems, listening on `[::]` (all IPv6 addresses) has a special
//! behavior: it will also accept IPv4 connections on the same port. There is system-wide
//! configuration for this behavior, e.g. `/proc/sys/net/ipv6/bindv6only` file on Linux.
//!
//! If you do not want the default system behavior, you can specify the `ipv6_only` flag
//! explicitly:
//!
//! ```yaml
//! listen:
//! - { addr: "[::]:8080", ipv6_only: true }
//! ```
//!
//! With the configuration above the server will still listen on all IPv6 addresses, yet IPv4
//! connections will only be accepted if configured explicitly.
//!
//! The `listen` configuration option is also available as `--listen` command line option. Flags
//! cannot be specified via the command line, only the address to listen on. This command line
//! option can be specified multiple times to make the server listen on multiple addresses or ports.
//!
//! Other command line options are: `--conf` (configuration file or configuration files to load),
//! `--daemon` (run process in background) and `--test` (test configuration and exit).
//!
//! ## TLS configuration
//!
//! You can enable TLS for some or all addresses the server listens on by specifying the `tls`
//! flag:
//!
//! ```yaml
//! listen:
//! - {addr: 127.0.0.1:8080, tls: true}
//! - {addr: "[::1]:8080", tls: true}
//! ```
//!
//! If TLS is used, the configuration at the very least has to specify the default certificate and
//! key:
//!
//! ```yaml
//! tls:
//!     cert_path: cert.pem
//!     key_path: key.pem
//! ```
//!
//! If you use different certificates for different server names (SNI), you can additionally list
//! these under `server_names`:
//!
//! ```yaml
//! tls:
//!     cert_path: cert.pem
//!     key_path: key.pem
//!     server_names:
//!         example.com:
//!             cert_path: cert.example.com.pem
//!             key_path: key.example.com.pem
//!         example.net:
//!             cert_path: cert.example.net.pem
//!             key_path: key.example.net.pem
//! ```
//!
//! If a server name indicator is received and a matching server name exists in the configuration,
//! the corresponding certificate will be used. Otherwise the default certificate will be used as
//! fallback.
//!
//! ## Code example
//!
//! ```rust
//! use async_trait::async_trait;
//! use module_utils::pingora::{Error, HttpPeer, ProxyHttp, Session};
//! use module_utils::FromYaml;
//! use startup_module::{StartupConf, StartupOpt};
//! use structopt::StructOpt;
//!
//! pub struct MyServer;
//!
//! #[async_trait]
//! impl ProxyHttp for MyServer {
//!     type CTX = ();
//!     fn new_ctx(&self) -> Self::CTX {}
//!
//!     async fn upstream_peer(
//!         &self,
//!         _session: &mut Session,
//!         _ctx: &mut Self::CTX,
//!     ) -> Result<Box<HttpPeer>, Box<Error>> {
//!         Ok(Box::new(HttpPeer::new(("example.com", 443), true, "example.com".to_owned())))
//!     }
//! }
//!
//! let opt = StartupOpt::from_args();
//! let conf = StartupConf::load_from_files(opt.conf.as_deref().unwrap_or(&[])).unwrap();
//! let server = conf.into_server(MyServer {}, Some(opt)).unwrap();
//!
//! // Do something with the server here, e.g. call server.run_forever()
//! ```
//!
//! For more comprehensive examples see the `examples` directory in the repository.

mod configuration;

use async_trait::async_trait;
pub use configuration::{CertKeyConf, ListenAddr, StartupConf, StartupOpt, TlsConf};
use module_utils::pingora::{Error, HttpPeer, ProxyHttp, ResponseHeader, Session};
use module_utils::RequestFilter;

/// A trivial Pingora app implementation, to be passed to [`StartupConf::into_server`]
///
/// This app will only handle the `early_request_filter`, `request_filter`, `upstream_peer`,
/// `upstream_response_filter` and `logging` phases. All processing will be delegated to the
/// respective `RequestFilter` methods.
#[derive(Debug, Clone)]
pub struct DefaultApp<H> {
    handler: H,
}

impl<H> DefaultApp<H> {
    /// Creates a new app from a [`RequestFilter`] instance.
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    /// Creates a new app from a [`RequestFilter`] configuration.
    ///
    /// Any errors occurring when converting configuration to handler will be passed on.
    pub fn from_conf<C>(conf: C) -> Result<Self, Box<Error>>
    where
        H: RequestFilter<Conf = C> + TryFrom<C, Error = Box<Error>>,
    {
        Ok(Self::new(conf.try_into()?))
    }
}

#[async_trait]
impl<H> ProxyHttp for DefaultApp<H>
where
    H: RequestFilter + Sync,
    H::CTX: Send,
{
    type CTX = <H as RequestFilter>::CTX;

    fn new_ctx(&self) -> Self::CTX {
        H::new_ctx()
    }

    async fn early_request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<(), Box<Error>> {
        self.handler.call_early_request_filter(session, ctx).await
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<bool, Box<Error>> {
        self.handler.call_request_filter(session, ctx).await
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>, Box<Error>> {
        self.handler.call_upstream_peer(session, ctx).await
    }

    fn upstream_response_filter(
        &self,
        session: &mut Session,
        response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) {
        self.handler.call_response_filter(session, response, ctx)
    }

    async fn logging(&self, session: &mut Session, e: Option<&Error>, ctx: &mut Self::CTX) {
        self.handler.call_logging(session, e, ctx).await
    }
}
