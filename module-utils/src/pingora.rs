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

//! Exposes some types from `pingora-core` and `pingora-proxy` crates, so that typical modules no
//! longer need them as direct dependencies.

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use http::{header, Extensions};
pub use pingora::http::{IntoCaseHeaderName, RequestHeader, ResponseHeader};
pub use pingora::modules::http::compression::ResponseCompression;
use pingora::modules::http::compression::ResponseCompressionBuilder;
use pingora::modules::http::HttpModules;
pub use pingora::protocols::http::HttpTask;
pub use pingora::protocols::l4::socket::SocketAddr;
pub use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
pub use pingora::server::configuration::{Opt as ServerOpt, ServerConf};
pub use pingora::server::Server;
pub use pingora::upstreams::peer::HttpPeer;
pub use pingora::{Error, ErrorType};
use std::borrow::Cow;
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

use crate::RequestFilter;

/// A trait implemented by wrappers around Pingora’s session
///
/// All the usual methods and fields of [`Session`] are available as well.
#[async_trait]
pub trait SessionWrapper: Send + Deref<Target = Session> + DerefMut {
    /// Attempts to determine the request host if one was specified.
    fn host(&self) -> Option<Cow<'_, str>>
    where
        Self: Sized,
    {
        fn host_from_header(session: &impl SessionWrapper) -> Option<Cow<'_, str>> {
            let host = session.get_header(header::HOST)?;
            host.to_str().ok().map(|h| h.into())
        }

        fn host_from_uri(session: &impl SessionWrapper) -> Option<Cow<'_, str>> {
            let uri = &session.req_header().uri;
            let host = uri.host()?;
            if let Some(port) = uri.port() {
                let mut host = host.to_owned();
                host.push(':');
                host.push_str(port.as_str());
                Some(host.into())
            } else {
                Some(host.into())
            }
        }

        host_from_header(self).or_else(|| host_from_uri(self))
    }

    /// Return the client (peer) address of the connection.
    ///
    /// Unlike the identical method of the Pingora session, this value can be overwritten.
    fn client_addr(&self) -> Option<&SocketAddr> {
        let addr = self.extensions().get();
        if addr.is_some() {
            addr
        } else {
            self.deref().client_addr()
        }
    }

    /// Overwrites the client address for this connection.
    fn set_client_addr(&mut self, addr: SocketAddr) {
        self.extensions_mut().insert(addr);
    }

    /// Returns a reference to the associated extensions.
    ///
    /// *Note*: The extensions are only present for the lifetime of the wrapper. Unlike `Session`
    /// or `CTX` data, they don’t survive across Pingora phases.
    fn extensions(&self) -> &Extensions;

    /// Returns a mutable reference to the associated extensions.
    ///
    /// *Note*: The extensions are only present for the lifetime of the wrapper. Unlike `Session`
    /// or `CTX` data, they don’t survive across Pingora phases.
    fn extensions_mut(&mut self) -> &mut Extensions;

    /// See [`Session::write_response_header`](pingora::protocols::http::server::Session::write_response_header)
    async fn write_response_header(
        &mut self,
        resp: Box<ResponseHeader>,
        end_of_stream: bool,
    ) -> Result<(), Box<Error>> {
        self.deref_mut()
            .write_response_header(resp, end_of_stream)
            .await
    }

    /// See [`Session::write_response_header_ref`](pingora::protocols::http::server::Session::write_response_header_ref)
    #[deprecated(
        note = "Please use write_response_header for now, see https://github.com/cloudflare/pingora/issues/206#issuecomment-2168764571"
    )]
    async fn write_response_header_ref(
        &mut self,
        _resp: &ResponseHeader,
    ) -> Result<(), Box<Error>> {
        // Looks like this cannot currently be made to work correctly, we don’t know whether this
        // is the end of the stream. See:
        // https://github.com/cloudflare/pingora/issues/206#issuecomment-2168764571
        panic!("Please use write_response_header, write_response_header_ref won't work correctly");
    }

    /// See [`Session::response_written`](pingora::protocols::http::server::Session::response_written)
    fn response_written(&self) -> Option<&ResponseHeader> {
        self.deref().response_written()
    }

    /// See [`Session::write_response_body`](pingora::protocols::http::server::Session::write_response_body)
    async fn write_response_body(
        &mut self,
        body: Option<Bytes>,
        end_of_stream: bool,
    ) -> Result<(), Box<Error>> {
        self.deref_mut()
            .write_response_body(body, end_of_stream)
            .await
    }
}

struct SessionWrapperImpl<'a, H> {
    inner: &'a mut Session,
    handler: &'a H,
    extensions: Extensions,
}

impl<'a, H> SessionWrapperImpl<'a, H> {
    fn from(inner: &'a mut Session, handler: &'a H) -> Self
    where
        H: RequestFilter,
    {
        Self {
            inner,
            handler,
            extensions: Extensions::new(),
        }
    }
}

#[async_trait]
impl<H> SessionWrapper for SessionWrapperImpl<'_, H>
where
    H: RequestFilter,
    for<'a> &'a H: Send,
{
    fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    async fn write_response_header(
        &mut self,
        mut resp: Box<ResponseHeader>,
        end_of_stream: bool,
    ) -> Result<(), Box<Error>> {
        self.handler.response_filter(self, &mut resp, None);

        self.deref_mut()
            .write_response_header(resp, end_of_stream)
            .await
    }
}

impl<H> Deref for SessionWrapperImpl<'_, H> {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<H> DerefMut for SessionWrapperImpl<'_, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

/// Creates a new session wrapper for the given Pingora session.
pub(crate) fn wrap_session<'a, H>(
    session: &'a mut Session,
    handler: &'a H,
) -> impl SessionWrapper + 'a
where
    H: RequestFilter + Sized + Sync,
{
    SessionWrapperImpl::from(session, handler)
}

/// A `SessionWrapper` implementation used for tests.
pub struct TestSession {
    inner: Session,
    extensions: Extensions,

    /// Set to `true` if end of the response body was sent
    pub end_of_stream: bool,

    /// The response header written if any
    pub response_header: Option<ResponseHeader>,

    /// The response body written if any
    pub response_body: BytesMut,
}

impl TestSession {
    /// Creates a new test session based with the given header.
    pub async fn from(header: RequestHeader) -> Self {
        Self::with_body(header, "").await
    }

    /// Creates a new test session based with the given header and request body.
    pub async fn with_body(mut header: RequestHeader, body: impl AsRef<[u8]>) -> Self {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let _ = cursor.write(b"POST / HTTP/1.1\r\n");
        let _ = cursor.write(b"Connection: close\r\n");
        let _ = cursor.write(b"\r\n");
        let _ = cursor.write(body.as_ref());
        let _ = cursor.seek(SeekFrom::Start(0));

        let _ = header.insert_header(header::CONTENT_LENGTH, body.as_ref().len());

        let mut modules = HttpModules::new();
        modules.add_module(ResponseCompressionBuilder::enable(0));

        let mut inner = Session::new_h1_with_modules(Box::new(cursor), &modules);
        assert!(inner.read_request().await.unwrap());
        *inner.req_header_mut() = header;

        Self {
            inner,
            extensions: Extensions::new(),
            end_of_stream: false,
            response_header: None,
            response_body: BytesMut::new(),
        }
    }
}

#[async_trait]
impl SessionWrapper for TestSession {
    fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    async fn write_response_header(
        &mut self,
        resp: Box<ResponseHeader>,
        end_of_stream: bool,
    ) -> Result<(), Box<Error>> {
        if self.response_header.is_some() {
            panic!("Trying to send a response header twice");
        }
        if self.end_of_stream {
            panic!("Trying to send a response header after end of stream");
        }
        self.end_of_stream = end_of_stream;
        self.response_header = Some(*resp);
        Ok(())
    }

    fn response_written(&self) -> Option<&ResponseHeader> {
        self.response_header.as_ref()
    }

    async fn write_response_body(
        &mut self,
        body: Option<Bytes>,
        end_of_stream: bool,
    ) -> Result<(), Box<Error>> {
        if self.end_of_stream {
            panic!("Trying to write response body after end of stream");
        }
        self.end_of_stream = end_of_stream;
        if let Some(body) = body {
            self.response_body.extend(std::iter::once(body));
        }
        Ok(())
    }
}

impl Deref for TestSession {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl std::fmt::Debug for TestSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSession").finish()
    }
}
