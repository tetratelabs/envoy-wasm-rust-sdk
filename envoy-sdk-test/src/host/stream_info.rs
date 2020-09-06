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

//! Fake `Stream Info API`.

use std::time::{Duration, SystemTime};

use envoy::extension::access_logger;
use envoy::host::stream_info::{ResponseFlags, StreamInfo, TrafficDirection};
use envoy::host::{self, ByteString, HeaderMap};

use crate::host::http::FakeHttpMessage;

/// Represents fake `Stream Info`.
#[derive(Debug, Default, Clone)]
pub struct FakeStreamInfo {
    connection: Option<FakeConnectionInfo>,
    request: Option<FakeRequestInfo>,
    response: Option<FakeResponseInfo>,
    upstream: Option<FakeUpstreamInfo>,
    source: Option<FakePeerInfo>,
    destination: Option<FakePeerInfo>,
    listener: Option<FakeListenerInfo>,
    route: Option<FakeRouteInfo>,
    cluster: Option<FakeClusterInfo>,
    plugin: Option<FakePluginInfo>,
}

/// Represents `connection` info.
#[derive(Debug, Default, Clone)]
struct FakeConnectionInfo {
    id: u64,
    requested_server_name: String,
    tls: Option<FakeTlsInfo>,
}

/// Represents `TLS` info.
#[derive(Debug, Default, Clone)]
struct FakeTlsInfo {
    version: Option<String>,
    subject_local_certificate: Option<String>,
    subject_peer_certificate: Option<String>,
    uri_san_local_certificate: Option<String>,
    uri_san_peer_certificate: Option<String>,
    dns_san_local_certificate: Option<String>,
    dns_san_peer_certificate: Option<String>,
}

/// Represents `request` info.
#[derive(Debug, Default, Clone)]
struct FakeRequestInfo {
    message: FakeHttpMessage,
    protocol: Option<String>,
    id: Option<String>,
    time: Option<SystemTime>,
    duration: Option<Duration>,
    size: u64,
    total_size: u64,
}

/// Represents `response` info.
#[derive(Debug, Default, Clone)]
struct FakeResponseInfo {
    message: FakeHttpMessage,
    size: u64,
    total_size: u64,
    flags: ResponseFlags,
}

/// Represents `upstream` info.
#[derive(Debug, Default, Clone)]
struct FakeUpstreamInfo {
    address: String,
    port: u32,
    local_address: Option<String>,
    transport_failure_reason: Option<String>,
    tls: Option<FakeTlsInfo>,
}

/// Represents info about connection `source` or `destination`.
#[derive(Debug, Default, Clone)]
struct FakePeerInfo {
    address: String,
    port: u32,
}

/// Represents `listener` info.
#[derive(Debug, Default, Clone)]
struct FakeListenerInfo {
    traffic_direction: TrafficDirection,
}

/// Represents `route` info.
#[derive(Debug, Default, Clone)]
struct FakeRouteInfo {
    name: String,
}

/// Represents `cluster` info.
#[derive(Debug, Default, Clone)]
struct FakeClusterInfo {
    name: String,
}

/// Represents `plugin` info.
#[derive(Debug, Default, Clone)]
struct FakePluginInfo {
    name: String,
    root_id: String,
    vm_id: String,
}

pub struct FakeConnectionInfoBuilder<'a> {
    connection: &'a mut Option<FakeConnectionInfo>,
}

pub struct FakeTlsInfoBuilder<'a> {
    tls: &'a mut Option<FakeTlsInfo>,
}

pub struct FakeRequestInfoBuilder<'a> {
    request: &'a mut Option<FakeRequestInfo>,
}

pub struct FakeResponseInfoBuilder<'a> {
    response: &'a mut Option<FakeResponseInfo>,
}

pub struct FakeUpstreamInfoBuilder<'a> {
    upstream: &'a mut Option<FakeUpstreamInfo>,
}

pub struct FakeSourceInfoBuilder<'a> {
    source: &'a mut Option<FakePeerInfo>,
}

pub struct FakeDestinationInfoBuilder<'a> {
    destination: &'a mut Option<FakePeerInfo>,
}

pub struct FakeListenerInfoBuilder<'a> {
    listener: &'a mut Option<FakeListenerInfo>,
}

pub struct FakeRouteInfoBuilder<'a> {
    route: &'a mut Option<FakeRouteInfo>,
}

pub struct FakeClusterInfoBuilder<'a> {
    cluster: &'a mut Option<FakeClusterInfo>,
}

pub struct FakePluginInfoBuilder<'a> {
    plugin: &'a mut Option<FakePluginInfo>,
}

impl FakeStreamInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, builder: impl FnOnce(&mut Self)) -> Self {
        builder(&mut self);
        self
    }

    pub fn connection(&mut self) -> FakeConnectionInfoBuilder<'_> {
        FakeConnectionInfoBuilder {
            connection: &mut self.connection,
        }
    }

    pub fn request(&mut self) -> FakeRequestInfoBuilder<'_> {
        FakeRequestInfoBuilder {
            request: &mut self.request,
        }
    }

    pub fn response(&mut self) -> FakeResponseInfoBuilder<'_> {
        FakeResponseInfoBuilder {
            response: &mut self.response,
        }
    }

    pub fn upstream(&mut self) -> FakeUpstreamInfoBuilder<'_> {
        FakeUpstreamInfoBuilder {
            upstream: &mut self.upstream,
        }
    }

    pub fn source(&mut self) -> FakeSourceInfoBuilder<'_> {
        FakeSourceInfoBuilder {
            source: &mut self.source,
        }
    }

    pub fn destination(&mut self) -> FakeDestinationInfoBuilder<'_> {
        FakeDestinationInfoBuilder {
            destination: &mut self.destination,
        }
    }

    pub fn listener(&mut self) -> FakeListenerInfoBuilder<'_> {
        FakeListenerInfoBuilder {
            listener: &mut self.listener,
        }
    }

    pub fn route(&mut self) -> FakeRouteInfoBuilder<'_> {
        FakeRouteInfoBuilder {
            route: &mut self.route,
        }
    }

    pub fn cluster(&mut self) -> FakeClusterInfoBuilder<'_> {
        FakeClusterInfoBuilder {
            cluster: &mut self.cluster,
        }
    }

    pub fn plugin(&mut self) -> FakePluginInfoBuilder<'_> {
        FakePluginInfoBuilder {
            plugin: &mut self.plugin,
        }
    }
}

impl FakeTlsInfo {
    fn has_local_cert(&self) -> bool {
        self.subject_local_certificate.is_some()
            || self.uri_san_local_certificate.is_some()
            || self.dns_san_local_certificate.is_some()
    }

    fn has_peer_cert(&self) -> bool {
        self.subject_peer_certificate.is_some()
            || self.uri_san_peer_certificate.is_some()
            || self.dns_san_peer_certificate.is_some()
    }

    fn is_mtls(&self) -> bool {
        self.has_local_cert() && self.has_peer_cert()
    }
}

impl FakeRequestInfo {
    fn url_path(&self) -> host::Result<Option<String>> {
        if let Some(path) = self.message.headers.get(":path") {
            let path = path.clone().into_string()?;
            let path: Vec<&str> = path.splitn(2, '?').collect();
            Ok(path[0].to_owned()).map(Option::from)
        } else {
            Ok(None)
        }
    }
}

impl FakeResponseInfo {
    fn status_code(&self) -> host::Result<Option<u16>> {
        if let Some(status) = self.message.headers.get(":status") {
            let status = status.clone().into_string()?;
            Ok(status.parse::<u16>()?).map(Option::from)
        } else {
            Ok(None)
        }
    }

    fn grpc_status(&self) -> host::Result<Option<i32>> {
        if let Some(status) = self.message.trailers.get("grpc-status") {
            let status = status.clone().into_string()?;
            Ok(status.parse::<i32>()?).map(Option::from)
        } else {
            Ok(None)
        }
    }
}

impl<'a> FakeConnectionInfoBuilder<'a> {
    pub fn id(&mut self, value: u64) -> &mut Self {
        self.connection.get_or_insert_with(Default::default).id = value;
        self
    }

    pub fn requested_server_name<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.connection
            .get_or_insert_with(Default::default)
            .requested_server_name = value.as_ref().to_owned();
        self
    }

    pub fn tls(&mut self) -> FakeTlsInfoBuilder<'_> {
        FakeTlsInfoBuilder {
            tls: &mut self.connection.get_or_insert_with(Default::default).tls,
        }
    }
}

impl<'a> FakeTlsInfoBuilder<'a> {
    pub fn version<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls.get_or_insert_with(Default::default).version = Some(value.as_ref().to_owned());
        self
    }

    pub fn subject_local_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .subject_local_certificate = Some(value.as_ref().to_owned());
        self
    }

    pub fn subject_peer_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .subject_peer_certificate = Some(value.as_ref().to_owned());
        self
    }

    pub fn uri_san_local_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .uri_san_local_certificate = Some(value.as_ref().to_owned());
        self
    }

    pub fn uri_san_peer_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .uri_san_peer_certificate = Some(value.as_ref().to_owned());
        self
    }

    pub fn dns_san_local_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .dns_san_local_certificate = Some(value.as_ref().to_owned());
        self
    }

    pub fn dns_san_peer_certificate<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.tls
            .get_or_insert_with(Default::default)
            .dns_san_peer_certificate = Some(value.as_ref().to_owned());
        self
    }
}

impl<'a> FakeRequestInfoBuilder<'a> {
    pub fn header<K, V>(&mut self, name: K, value: V) -> &mut Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.request
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(name, value);
        self
    }

    pub fn method<V>(&mut self, value: V) -> &mut Self
    where
        V: AsRef<str>,
    {
        self.request
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(":method", value.as_ref());
        self
    }

    pub fn scheme<V>(&mut self, value: V) -> &mut Self
    where
        V: AsRef<str>,
    {
        self.request
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(":scheme", value.as_ref());
        self
    }

    pub fn host<V>(&mut self, value: V) -> &mut Self
    where
        V: AsRef<str>,
    {
        self.request
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(":authority", value.as_ref());
        self
    }

    pub fn path<V>(&mut self, value: V) -> &mut Self
    where
        V: AsRef<str>,
    {
        self.request
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(":path", value.as_ref());
        self
    }

    pub fn protocol<V>(&mut self, value: V) -> &mut Self
    where
        V: AsRef<str>,
    {
        self.request.get_or_insert_with(Default::default).protocol = Some(value.as_ref().into());
        self
    }

    pub fn id<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.request.get_or_insert_with(Default::default).id = Some(value.as_ref().into());
        self
    }

    pub fn time(&mut self, value: SystemTime) -> &mut Self {
        self.request.get_or_insert_with(Default::default).time = Some(value);
        self
    }

    pub fn duration(&mut self, value: Duration) -> &mut Self {
        self.request.get_or_insert_with(Default::default).duration = Some(value);
        self
    }

    pub fn size(&mut self, value: u64) -> &mut Self {
        self.request.get_or_insert_with(Default::default).size = value;
        self
    }

    pub fn total_size(&mut self, value: u64) -> &mut Self {
        self.request.get_or_insert_with(Default::default).total_size = value;
        self
    }
}

impl<'a> FakeResponseInfoBuilder<'a> {
    pub fn status_code(&mut self, value: u16) -> &mut Self {
        self.response
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(":status", value.to_string());
        self
    }

    pub fn header<K, V>(&mut self, name: K, value: V) -> &mut Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.response
            .get_or_insert_with(Default::default)
            .message
            .headers
            .insert(name.into(), value.into());
        self
    }

    pub fn trailer<K, V>(&mut self, name: K, value: V) -> &mut Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.response
            .get_or_insert_with(Default::default)
            .message
            .trailers
            .insert(name.into(), value.into());
        self
    }

    pub fn size(&mut self, value: u64) -> &mut Self {
        self.response.get_or_insert_with(Default::default).size = value;
        self
    }

    pub fn total_size(&mut self, value: u64) -> &mut Self {
        self.response
            .get_or_insert_with(Default::default)
            .total_size = value;
        self
    }

    pub fn response_flags(&mut self, value: ResponseFlags) -> &mut Self {
        self.response.get_or_insert_with(Default::default).flags = value;
        self
    }

    pub fn grpc_status(&mut self, value: i32) -> &mut Self {
        self.response
            .get_or_insert_with(Default::default)
            .message
            .trailers
            .insert("grpc-status", value.to_string());
        self
    }
}

impl<'a> FakeUpstreamInfoBuilder<'a> {
    pub fn address<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.upstream.get_or_insert_with(Default::default).address = value.as_ref().to_owned();
        self
    }

    pub fn port(&mut self, value: u32) -> &mut Self {
        self.upstream.get_or_insert_with(Default::default).port = value;
        self
    }

    pub fn local_address<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.upstream
            .get_or_insert_with(Default::default)
            .local_address = Some(value.as_ref().to_owned());
        self
    }

    pub fn transport_failure_reason<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.upstream
            .get_or_insert_with(Default::default)
            .transport_failure_reason = Some(value.as_ref().to_owned());
        self
    }

    pub fn tls(&mut self) -> FakeTlsInfoBuilder<'_> {
        FakeTlsInfoBuilder {
            tls: &mut self.upstream.get_or_insert_with(Default::default).tls,
        }
    }
}

impl<'a> FakeSourceInfoBuilder<'a> {
    pub fn address<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.source.get_or_insert_with(Default::default).address = value.as_ref().to_owned();
        self
    }

    pub fn port(&mut self, value: u32) -> &mut Self {
        self.source.get_or_insert_with(Default::default).port = value;
        self
    }
}

impl<'a> FakeDestinationInfoBuilder<'a> {
    pub fn address<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.destination
            .get_or_insert_with(Default::default)
            .address = value.as_ref().to_owned();
        self
    }

    pub fn port(&mut self, value: u32) -> &mut Self {
        self.destination.get_or_insert_with(Default::default).port = value;
        self
    }
}

impl<'a> FakeListenerInfoBuilder<'a> {
    pub fn traffic_direction(&mut self, value: TrafficDirection) -> &mut Self {
        self.listener
            .get_or_insert_with(Default::default)
            .traffic_direction = value;
        self
    }
}

impl<'a> FakeRouteInfoBuilder<'a> {
    pub fn name<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.route.get_or_insert_with(Default::default).name = value.as_ref().to_owned();
        self
    }
}

impl<'a> FakeClusterInfoBuilder<'a> {
    pub fn name<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.cluster.get_or_insert_with(Default::default).name = value.as_ref().to_owned();
        self
    }
}

impl<'a> FakePluginInfoBuilder<'a> {
    pub fn name<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.plugin.get_or_insert_with(Default::default).name = value.as_ref().to_owned();
        self
    }

    pub fn root_id<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.plugin.get_or_insert_with(Default::default).root_id = value.as_ref().to_owned();
        self
    }

    pub fn vm_id<T>(&mut self, value: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.plugin.get_or_insert_with(Default::default).vm_id = value.as_ref().to_owned();
        self
    }
}

impl StreamInfo for FakeStreamInfo {
    fn stream_property(&self, path: &[&str]) -> host::Result<Option<ByteString>> {
        let encoded = match path {
            // connection
            ["connection_id"] => self
                .connection
                .as_ref()
                .map(|con| con.id)
                .map(Encoder::encode_u64),
            ["connection", "mtls"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.is_mtls())
                .map(Encoder::encode_bool),
            ["connection", "requested_server_name"] => self
                .connection
                .as_ref()
                .map(|con| &con.requested_server_name)
                .map(Encoder::encode_str),
            ["connection", "tls_version"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.version.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "subject_local_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.subject_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "subject_peer_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.subject_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "uri_san_local_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.uri_san_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "uri_san_peer_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.uri_san_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "dns_san_local_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.dns_san_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["connection", "dns_san_peer_certificate"] => self
                .connection
                .as_ref()
                .map(|con| con.tls.as_ref())
                .flatten()
                .map(|tls| tls.dns_san_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            // request
            ["request", "headers", name] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get(name))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "id"] => self
                .request
                .as_ref()
                .map(|request| request.id.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["request", "time"] => self
                .request
                .as_ref()
                .map(|request| request.time)
                .flatten()
                .map(Encoder::encode_timestamp),
            ["request", "duration"] => self
                .request
                .as_ref()
                .map(|request| request.duration)
                .flatten()
                .map(Encoder::encode_duration),
            ["request", "size"] => self
                .request
                .as_ref()
                .map(|request| request.size as i64)
                .map(Encoder::encode_i64),
            ["request", "total_size"] => self
                .request
                .as_ref()
                .map(|request| request.total_size as i64)
                .map(Encoder::encode_i64),
            ["request", "method"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get(":method"))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "scheme"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get(":scheme"))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "host"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get(":authority"))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "path"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get(":path"))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "url_path"] => self
                .request
                .as_ref()
                .map(|request| request.url_path())
                .transpose()?
                .flatten()
                .map(Encoder::encode_str),
            ["request", "protocol"] => self
                .request
                .as_ref()
                .map(|request| request.protocol.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["request", "useragent"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get("user-agent"))
                .flatten()
                .map(Encoder::encode_str),
            ["request", "referer"] => self
                .request
                .as_ref()
                .map(|request| request.message.headers.get("referer"))
                .flatten()
                .map(Encoder::encode_str),
            // response
            ["response", "headers", name] => self
                .response
                .as_ref()
                .map(|response| response.message.headers.get(name))
                .flatten()
                .map(Encoder::encode_str),
            ["response", "trailers", name] => self
                .response
                .as_ref()
                .map(|response| response.message.trailers.get(name))
                .flatten()
                .map(Encoder::encode_str),
            ["response", "code"] => self
                .response
                .as_ref()
                .map(|response| response.status_code())
                .transpose()?
                .flatten()
                .map(|status_code| status_code as i64)
                .map(Encoder::encode_i64),
            ["response", "size"] => self
                .response
                .as_ref()
                .map(|response| response.size as i64)
                .map(Encoder::encode_i64),
            ["response", "total_size"] => self
                .response
                .as_ref()
                .map(|response| response.total_size as i64)
                .map(Encoder::encode_i64),
            ["response", "flags"] => self
                .response
                .as_ref()
                .map(|response| response.flags.bits() as i64)
                .map(Encoder::encode_i64),
            ["response", "grpc_status"] => self
                .response
                .as_ref()
                .map(|response| response.grpc_status())
                .transpose()?
                .flatten()
                .map(|status_code| status_code as i64)
                .map(Encoder::encode_i64),
            // upstream
            ["upstream", "address"] => self
                .upstream
                .as_ref()
                .map(|upstream| &upstream.address)
                .map(Encoder::encode_str),
            ["upstream", "port"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.port as i64)
                .map(Encoder::encode_i64),
            ["upstream", "local_address"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.local_address.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "transport_failure_reason"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.transport_failure_reason.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "tls_version"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.version.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "subject_local_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.subject_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "subject_peer_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.subject_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "uri_san_local_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.uri_san_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "uri_san_peer_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.uri_san_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "dns_san_local_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.dns_san_local_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            ["upstream", "dns_san_peer_certificate"] => self
                .upstream
                .as_ref()
                .map(|upstream| upstream.tls.as_ref())
                .flatten()
                .map(|tls| tls.dns_san_peer_certificate.as_ref())
                .flatten()
                .map(Encoder::encode_str),
            // source
            ["source", "address"] => self
                .source
                .as_ref()
                .map(|source| &source.address)
                .map(Encoder::encode_str),
            ["source", "port"] => self
                .source
                .as_ref()
                .map(|source| source.port as i64)
                .map(Encoder::encode_i64),
            // destination
            ["destination", "address"] => self
                .destination
                .as_ref()
                .map(|destination| &destination.address)
                .map(Encoder::encode_str),
            ["destination", "port"] => self
                .destination
                .as_ref()
                .map(|destination| destination.port as i64)
                .map(Encoder::encode_i64),
            // listener
            ["listener_direction"] => self
                .listener
                .as_ref()
                .map(|listener| listener.traffic_direction as i64)
                .map(Encoder::encode_i64),
            // route
            ["route_name"] => self
                .route
                .as_ref()
                .map(|route| &route.name)
                .map(Encoder::encode_str),
            // cluster
            ["cluster_name"] => self
                .cluster
                .as_ref()
                .map(|cluster| &cluster.name)
                .map(Encoder::encode_str),
            // plugin
            ["plugin_name"] => self
                .plugin
                .as_ref()
                .map(|plugin| &plugin.name)
                .map(Encoder::encode_str),
            ["plugin_root_id"] => self
                .plugin
                .as_ref()
                .map(|plugin| &plugin.root_id)
                .map(Encoder::encode_str),
            ["plugin_vm_id"] => self
                .plugin
                .as_ref()
                .map(|plugin| &plugin.vm_id)
                .map(Encoder::encode_str),
            _ => None,
        };
        encoded.unwrap_or_else(|| Ok(None))
    }

    fn set_stream_property(&self, _path: &[&str], _value: &[u8]) -> host::Result<()> {
        Ok(())
    }
}

impl access_logger::LogOps for FakeStreamInfo {
    fn request_headers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .request
            .as_ref()
            .map(|request| request.message.headers.clone())
            .unwrap_or_else(Default::default))
    }

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .request
            .as_ref()
            .map(|request| request.message.headers.get(name).map(Clone::clone))
            .flatten())
    }

    fn response_headers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .response
            .as_ref()
            .map(|response| response.message.headers.clone())
            .unwrap_or_else(Default::default))
    }

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .response
            .as_ref()
            .map(|response| response.message.headers.get(name).map(Clone::clone))
            .flatten())
    }

    fn response_trailers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .response
            .as_ref()
            .map(|response| response.message.trailers.clone())
            .unwrap_or_else(Default::default))
    }

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .response
            .as_ref()
            .map(|response| response.message.trailers.get(name).map(Clone::clone))
            .flatten())
    }

    fn stream_info(&self) -> &dyn StreamInfo {
        self
    }
}

struct Encoder;

impl Encoder {
    pub fn encode_i64(value: i64) -> host::Result<Option<ByteString>> {
        Ok(Some(value.to_le_bytes().as_ref().into()))
    }

    pub fn encode_u64(value: u64) -> host::Result<Option<ByteString>> {
        Ok(Some(value.to_le_bytes().as_ref().into()))
    }

    pub fn encode_bool(value: bool) -> host::Result<Option<ByteString>> {
        Ok(Some((value as u8).to_le_bytes().as_ref().into()))
    }

    pub fn encode_str<T: AsRef<[u8]>>(value: T) -> host::Result<Option<ByteString>> {
        Ok(Some(value.as_ref().into()))
    }

    pub fn encode_timestamp(value: SystemTime) -> host::Result<Option<ByteString>> {
        let value = value.duration_since(SystemTime::UNIX_EPOCH)?;
        Self::encode_duration(value)
    }

    pub fn encode_duration(value: Duration) -> host::Result<Option<ByteString>> {
        let value = value.as_secs() * 1_000_000_000 + value.subsec_nanos() as u64;
        Self::encode_i64(value as i64)
    }
}
