// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `Envoy` `Stream Info API`.

use core::convert::{TryFrom, TryInto};
use std::time::{Duration, SystemTime};

use self::property::{
    Cluster, Connection, Destination, Listener, Plugin, Property, Request, Response, Route, Source,
    Upstream,
};
use crate::host::error::function;
use crate::host::{self, Bytes, HeaderValue};

pub use self::types::{ResponseFlags, TrafficDirection};

mod property;
mod proxy_wasm;
mod types;

/// An interface of the `Envoy` `Stream Info API`.
///
/// Basic usage of [`StreamInfo`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::StreamInfo;
///
/// let stream_info = StreamInfo::default();
///
/// let connection_id = stream_info.connection().id()?;
/// let request_id = stream_info.request().id()?;
/// let plugin_name = stream_info.plugin().name()?;
///
/// stream_info.set_stream_property(&["my_extension", "output"], b"property value")?;
/// # Ok(())
/// # }
/// ```
///
/// [`StreamInfo`]: trait.StreamInfo.html
pub trait StreamInfo {
    /// Evaluates value of a given property in the enclosing context.
    ///
    /// * In case [`HttpFilter`], the value will be evaluated in the context of HTTP stream.
    /// * In case [`NetworkFilter`], the value will be evaluated in the context of TCP connection.
    /// * In case [`AccessLogger`], the value will be evaluated in the context of HTTP stream
    ///   or TCP connection that is being logged.
    ///
    /// # Arguments
    ///
    /// * `path` - property path as an array of path segments
    ///
    /// [`HttpFilter`]: ../../extension/filter/http/trait.HttpFilter.html
    /// [`NetworkFilter`]: ../../extension/filter/network/trait.NetworkFilter.html
    /// [`AccessLogger`]: ../../extension/access_logger/trait.AccessLogger.html
    fn stream_property(&self, path: &[&str]) -> host::Result<Option<Bytes>>;

    /// Saves a value in the enclosing context.
    ///
    /// The value will be accessible to other filters on that HTTP stream or TCP connection.
    ///
    /// # Arguments
    ///
    /// * `path`  - property path as an array of path segments
    /// * `value` - an opaque blob of bytes
    fn set_stream_property(&self, path: &[&str], value: &[u8]) -> host::Result<()>;
}

impl dyn StreamInfo {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn StreamInfo {
        &impls::Host
    }
}

impl<'a> dyn StreamInfo + 'a {
    /// Provides access to `request` properties.
    pub fn request(&'a self) -> RequestInfo<'a> {
        RequestInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `response` properties.
    pub fn response(&'a self) -> ResponseInfo<'a> {
        ResponseInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `connection` properties.
    pub fn connection(&'a self) -> ConnectionInfo<'a> {
        ConnectionInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `upstream` properties.
    pub fn upstream(&'a self) -> UpstreamInfo<'a> {
        UpstreamInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `source` properties.
    pub fn source(&'a self) -> SourceInfo<'a> {
        SourceInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `destination` properties.
    pub fn destination(&'a self) -> DestinationInfo<'a> {
        DestinationInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `listener` properties.
    pub fn listener(&'a self) -> ListenerInfo<'a> {
        ListenerInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `route` properties.
    pub fn route(&'a self) -> RouteInfo<'a> {
        RouteInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `cluster` properties.
    pub fn cluster(&'a self) -> ClusterInfo<'a> {
        ClusterInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }

    /// Provides access to `plugin` properties.
    pub fn plugin(&'a self) -> PluginInfo<'a> {
        PluginInfo {
            stream: StreamInfoAccessor { stream_info: self },
        }
    }
}

/// Provides access to the properties of a stream.
struct StreamInfoAccessor<'a> {
    stream_info: &'a dyn StreamInfo,
}

impl<'a> StreamInfoAccessor<'a> {
    fn property<T, W>(&self, prop: &Property<T, W>) -> host::Result<Option<T>>
    where
        T: TryFrom<proxy_wasm::Value<W>, Error = host::Error>,
    {
        if let Some(bytes) = self.stream_info.stream_property(prop.path())? {
            let encoded = proxy_wasm::Value::<W>::new(bytes.into_vec());
            let decoded: host::Result<T> = encoded.try_into();
            decoded.map(Option::from).map_err(|err| {
                function("env", "proxy_get_property")
                    .into_parse_error(
                        format!(
                            "value of property \"{:?}\" is not valid: {:?}",
                            prop.path(),
                            err,
                        )
                        .into(),
                    )
                    .into()
            })
        } else {
            Ok(None)
        }
    }
}

/// Provides access to `request` properties.
pub struct RequestInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> RequestInfo<'a> {
    /// Returns request header by name.
    pub fn header<K>(&self, name: K) -> host::Result<Option<HeaderValue>>
    where
        K: AsRef<str>,
    {
        self.stream.property(&Request::header(name.as_ref()))
    }

    /// Returns request ID.
    pub fn id(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::ID)
    }

    /// Returns time of the first byte received.
    pub fn time(&self) -> host::Result<Option<SystemTime>> {
        self.stream.property(Request::TIME)
    }

    /// Returns total duration of the request.
    pub fn duration(&self) -> host::Result<Option<Duration>> {
        self.stream.property(Request::DURATION)
    }

    /// Returns size of the request body.
    pub fn size(&self) -> host::Result<Option<u64>> {
        self.stream.property(Request::SIZE)
    }

    /// Returns total size of the request including the headers.
    pub fn total_size(&self) -> host::Result<Option<u64>> {
        self.stream.property(Request::TOTAL_SIZE)
    }

    /// Returns request protocol e.g. "HTTP/2".
    pub fn protocol(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::PROTOCOL)
    }

    /// Returns the path portion of the URL.
    pub fn path(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::PATH)
    }

    /// Returns the path portion of the URL without the query string.
    pub fn url_path(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::URL_PATH)
    }

    /// Returns the host portion of the URL.
    pub fn host(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::HOST)
    }

    /// Returns request method.
    pub fn method(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::METHOD)
    }

    /// Returns the scheme portion of the URL.
    pub fn scheme(&self) -> host::Result<Option<String>> {
        self.stream.property(Request::SCHEME)
    }

    /// Returns referer request header.
    pub fn referer(&self) -> host::Result<Option<HeaderValue>> {
        self.stream.property(Request::REFERER)
    }

    /// Returns user agent request header.
    pub fn user_agent(&self) -> host::Result<Option<HeaderValue>> {
        self.stream.property(Request::USER_AGENT)
    }
}

/// Provides access to `response` properties.
pub struct ResponseInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> ResponseInfo<'a> {
    /// Returns response header by name.
    pub fn header<K>(&self, name: K) -> host::Result<Option<HeaderValue>>
    where
        K: AsRef<str>,
    {
        self.stream.property(&Response::header(name.as_ref()))
    }

    /// Returns response trailer by name.
    pub fn trailer<K>(&self, name: K) -> host::Result<Option<HeaderValue>>
    where
        K: AsRef<str>,
    {
        self.stream.property(&Response::trailer(name.as_ref()))
    }

    /// Returns response HTTP status code.
    pub fn status_code(&self) -> host::Result<Option<u16>> {
        self.stream.property(Response::STATUS_CODE)
    }

    /// Returns size of the response body.
    pub fn size(&self) -> host::Result<Option<u64>> {
        self.stream.property(Response::SIZE)
    }

    /// Returns total size of the response including the approximate uncompressed size of the headers and the trailers.
    pub fn total_size(&self) -> host::Result<Option<u64>> {
        self.stream.property(Response::TOTAL_SIZE)
    }

    /// Returns additional details about the response beyond the standard response code.
    pub fn flags(&self) -> host::Result<Option<ResponseFlags>> {
        self.stream.property(Response::FLAGS)
    }

    /// Returns response gRPC status code.
    pub fn grpc_status(&self) -> host::Result<Option<i32>> {
        self.stream.property(Response::GRPC_STATUS)
    }
}

/// Provides access to `connection` properties.
pub struct ConnectionInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> ConnectionInfo<'a> {
    /// Returns connection ID.
    pub fn id(&self) -> host::Result<Option<u64>> {
        self.stream.property(Connection::ID)
    }

    /// Returns whether TLS is applied to the downstream connection and the peer ceritificate is presented.
    pub fn is_mtls(&self) -> host::Result<Option<bool>> {
        self.stream.property(Connection::IS_MTLS)
    }

    /// Returns requested server name in the downstream TLS connection.
    pub fn requested_server_name(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::REQUESTED_SERVER_NAME)
    }

    /// Returns TLS version of the downstream TLS connection.
    pub fn tls_version(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::TLS_VERSION)
    }

    /// Returns the subject field of the local certificate in the downstream TLS connection..
    pub fn subject_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::SUBJECT_LOCAL_CERTIFICATE)
    }

    /// Returns the subject field of the peer certificate in the downstream TLS connection.
    pub fn subject_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::SUBJECT_PEER_CERTIFICATE)
    }

    /// Returns the first URI entry in the SAN field of the local certificate in the downstream TLS connection.
    pub fn uri_san_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::URI_SAN_LOCAL_CERTIFICATE)
    }

    /// Returns the first URI entry in the SAN field of the peer certificate in the downstream TLS connection.
    pub fn uri_san_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::URI_SAN_PEER_CERTIFICATE)
    }

    /// Returns the first DNS entry in the SAN field of the local certificate in the downstream TLS connection.
    pub fn dns_san_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::DNS_SAN_LOCAL_CERTIFICATE)
    }

    /// Returns the first DNS entry in the SAN field of the peer certificate in the downstream TLS connection.
    pub fn dns_san_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Connection::DNS_SAN_PEER_CERTIFICATE)
    }
}

/// Provides access to `upstream` properties.
pub struct UpstreamInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> UpstreamInfo<'a> {
    /// Returns upstream connection remote address.
    pub fn address(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::ADDRESS)
    }

    /// Returns upstream connection remote port.
    pub fn port(&self) -> host::Result<Option<u32>> {
        self.stream.property(Upstream::PORT)
    }

    /// Returns the local address of the upstream connection.
    pub fn local_address(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::LOCAL_ADDRESS)
    }

    /// Returns the upstream transport failure reason e.g. certificate validation failed.
    pub fn transport_failure_reason(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::TRANSPORT_FAILURE_REASON)
    }

    /// Returns TLS version of the upstream TLS connection.
    pub fn tls_version(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::TLS_VERSION)
    }

    /// Returns the subject field of the local certificate in the upstream TLS connection.
    pub fn subject_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::SUBJECT_LOCAL_CERTIFICATE)
    }

    /// Returns the subject field of the peer certificate in the upstream TLS connection.
    pub fn subject_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::SUBJECT_PEER_CERTIFICATE)
    }

    /// Returns the first URI entry in the SAN field of the local certificate in the upstream TLS connection.
    pub fn uri_san_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::URI_SAN_LOCAL_CERTIFICATE)
    }

    /// Returns the first URI entry in the SAN field of the peer certificate in the upstream TLS connection.
    pub fn uri_san_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::URI_SAN_PEER_CERTIFICATE)
    }

    /// Returns the first DNS entry in the SAN field of the local certificate in the upstream TLS connection.
    pub fn dns_san_local_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::DNS_SAN_LOCAL_CERTIFICATE)
    }

    /// Returns the first DNS entry in the SAN field of the peer certificate in the upstream TLS connection.
    pub fn dns_san_peer_certificate(&self) -> host::Result<Option<String>> {
        self.stream.property(Upstream::DNS_SAN_PEER_CERTIFICATE)
    }
}

/// Provides access to `source` properties.
pub struct SourceInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> SourceInfo<'a> {
    /// Returns downstream connection remote address.
    pub fn address(&self) -> host::Result<Option<String>> {
        self.stream.property(Source::ADDRESS)
    }

    /// Returns downstream connection remote port.
    pub fn port(&self) -> host::Result<Option<u32>> {
        self.stream.property(Source::PORT)
    }
}

/// Provides access to `destination` properties.
pub struct DestinationInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> DestinationInfo<'a> {
    /// Returns downstream connection local address.
    pub fn address(&self) -> host::Result<Option<String>> {
        self.stream.property(Destination::ADDRESS)
    }

    /// Returns downstream connection local port.
    pub fn port(&self) -> host::Result<Option<u32>> {
        self.stream.property(Destination::PORT)
    }
}

/// Provides access to `listener` properties.
pub struct ListenerInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> ListenerInfo<'a> {
    /// Returns traffic direction.
    pub fn traffic_direction(&self) -> host::Result<Option<TrafficDirection>> {
        self.stream.property(Listener::TRAFFIC_DIRECTION)
    }
}

/// Provides access to `cluster` properties.
pub struct ClusterInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> ClusterInfo<'a> {
    /// Returns cluster name.
    pub fn name(&self) -> host::Result<Option<String>> {
        self.stream.property(Cluster::NAME)
    }
}

/// Provides access to `route` properties.
pub struct RouteInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> RouteInfo<'a> {
    /// Returns route name.
    pub fn name(&self) -> host::Result<Option<String>> {
        self.stream.property(Route::NAME)
    }
}

/// Provides access to `plugin` properties.
pub struct PluginInfo<'a> {
    stream: StreamInfoAccessor<'a>,
}

impl<'a> PluginInfo<'a> {
    /// Returns plugin name.
    pub fn name(&self) -> host::Result<Option<String>> {
        self.stream.property(Plugin::NAME)
    }

    /// Returns plugin Root ID.
    pub fn root_id(&self) -> host::Result<Option<String>> {
        self.stream.property(Plugin::ROOT_ID)
    }

    /// Returns plugin VM ID.
    pub fn vm_id(&self) -> host::Result<Option<String>> {
        self.stream.property(Plugin::VM_ID)
    }
}

mod impls {
    use crate::abi::proxy_wasm::hostcalls;

    use super::StreamInfo;
    use crate::host::{self, Bytes};

    pub(super) struct Host;

    impl StreamInfo for Host {
        fn stream_property(&self, path: &[&str]) -> host::Result<Option<Bytes>> {
            hostcalls::get_property(path)
        }

        fn set_stream_property(&self, path: &[&str], value: &[u8]) -> host::Result<()> {
            hostcalls::set_property(path, value)
        }
    }
}
