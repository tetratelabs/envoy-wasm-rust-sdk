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

use std::time::{Duration, SystemTime};

use envoy::host::stream_info::{ResponseFlags, TrafficDirection};
use envoy::host::{HeaderMap, Result, StreamInfo};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeStreamInfo;

#[test]
fn test_connection() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.connection()
            .id(123)
            .requested_server_name("example.org")
            .tls()
            .version("TLSv1.2")
            .subject_local_certificate("CN=gateway")
            .subject_peer_certificate("CN=downstream")
            .uri_san_local_certificate("spiffe://cluster.local/gateway")
            .uri_san_peer_certificate("spiffe://cluster.local/downstream")
            .dns_san_local_certificate("gateway.svc")
            .dns_san_peer_certificate("downstream.svc");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.connection().id()?, Some(123));
    assert_eq!(stream_info.connection().is_mtls()?, Some(true));
    assert_eq!(
        stream_info.connection().requested_server_name()?,
        Some("example.org".to_owned())
    );
    assert_eq!(
        stream_info.connection().tls().version()?,
        Some("TLSv1.2".into())
    );
    assert_eq!(
        stream_info.connection().tls().subject_local_certificate()?,
        Some("CN=gateway".into())
    );
    assert_eq!(
        stream_info.connection().tls().subject_peer_certificate()?,
        Some("CN=downstream".into())
    );
    assert_eq!(
        stream_info.connection().tls().uri_san_local_certificate()?,
        Some("spiffe://cluster.local/gateway".into())
    );
    assert_eq!(
        stream_info.connection().tls().uri_san_peer_certificate()?,
        Some("spiffe://cluster.local/downstream".into())
    );
    assert_eq!(
        stream_info.connection().tls().dns_san_local_certificate()?,
        Some("gateway.svc".into())
    );
    assert_eq!(
        stream_info.connection().tls().dns_san_peer_certificate()?,
        Some("downstream.svc".into())
    );

    Ok(())
}

#[test]
fn test_http_request() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.request()
            .id("a-b-c-d")
            .time(SystemTime::UNIX_EPOCH + Duration::from_secs(12))
            .duration(Duration::from_millis(123))
            .size(1024)
            .total_size(2048)
            .method("GET")
            .scheme("https")
            .host("www.example.com")
            .path("/search?q=example")
            .protocol("HTTP/1.1")
            .header("content-type", "application/json")
            .header("content-length", "1001")
            .header("user-agent", "curl")
            .header("referer", "https://www.example.com");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.request().id()?, Some("a-b-c-d".to_owned()));
    assert_eq!(
        stream_info.request().time()?,
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(12))
    );
    assert_eq!(
        stream_info.request().duration()?,
        Some(Duration::from_millis(123))
    );
    assert_eq!(stream_info.request().size()?, Some(1024));
    assert_eq!(stream_info.request().total_size()?, Some(2048));
    assert_eq!(stream_info.request().method()?, Some("GET".into()));
    assert_eq!(stream_info.request().scheme()?, Some("https".into()));
    assert_eq!(
        stream_info.request().host()?,
        Some("www.example.com".into())
    );
    assert_eq!(
        stream_info.request().path()?,
        Some("/search?q=example".into())
    );
    assert_eq!(stream_info.request().url_path()?, Some("/search".into()));
    assert_eq!(stream_info.request().protocol()?, Some("HTTP/1.1".into()));
    assert_eq!(
        stream_info.request().header("content-length")?,
        Some("1001".into())
    );
    assert_eq!(stream_info.request().user_agent()?, Some("curl".into()));
    assert_eq!(
        stream_info.request().referer()?,
        Some("https://www.example.com".into())
    );

    Ok(())
}

#[test]
fn test_http_response() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.response()
            .status_code(201)
            .header("content-type", "application/json")
            .header("content-length", "1001")
            .trailer("grpc-message", "UNKNOWN")
            .size(1024)
            .total_size(2048)
            .grpc_status(1)
            .response_flags(
                ResponseFlags::FAILED_LOCAL_HEALTH_CHECK | ResponseFlags::DELAY_INJECTED,
            );
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.response().status_code()?, Some(201));
    assert_eq!(stream_info.response().size()?, Some(1024));
    assert_eq!(stream_info.response().total_size()?, Some(2048));
    assert_eq!(stream_info.response().grpc_status()?, Some(1));
    assert_eq!(
        stream_info.response().flags()?,
        Some(ResponseFlags::FAILED_LOCAL_HEALTH_CHECK | ResponseFlags::DELAY_INJECTED)
    );
    assert_eq!(
        stream_info.response().header("content-type")?,
        Some("application/json".into())
    );
    assert_eq!(
        stream_info.response().trailer("grpc-message")?,
        Some("UNKNOWN".into())
    );

    Ok(())
}

#[test]
fn test_upstream() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.upstream()
            .address("192.168.0.1")
            .port(5432)
            .local_address("127.0.0.1")
            .transport_failure_reason("bad luck")
            .tls()
            .version("TLSv1.1")
            .subject_local_certificate("CN=gateway")
            .subject_peer_certificate("CN=upstream")
            .uri_san_local_certificate("spiffe://cluster.local/gateway")
            .uri_san_peer_certificate("spiffe://cluster.local/upstream")
            .dns_san_local_certificate("gateway.svc")
            .dns_san_peer_certificate("upstream.svc");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(
        stream_info.upstream().address()?,
        Some("192.168.0.1".to_owned())
    );
    assert_eq!(stream_info.upstream().port()?, Some(5432));
    assert_eq!(
        stream_info.upstream().local_address()?,
        Some("127.0.0.1".to_owned())
    );
    assert_eq!(
        stream_info.upstream().transport_failure_reason()?,
        Some("bad luck".to_owned())
    );
    assert_eq!(
        stream_info.upstream().tls().version()?,
        Some("TLSv1.1".into())
    );
    assert_eq!(
        stream_info.upstream().tls().subject_local_certificate()?,
        Some("CN=gateway".into())
    );
    assert_eq!(
        stream_info.upstream().tls().subject_peer_certificate()?,
        Some("CN=upstream".into())
    );
    assert_eq!(
        stream_info.upstream().tls().uri_san_local_certificate()?,
        Some("spiffe://cluster.local/gateway".into())
    );
    assert_eq!(
        stream_info.upstream().tls().uri_san_peer_certificate()?,
        Some("spiffe://cluster.local/upstream".into())
    );
    assert_eq!(
        stream_info.upstream().tls().dns_san_local_certificate()?,
        Some("gateway.svc".into())
    );
    assert_eq!(
        stream_info.upstream().tls().dns_san_peer_certificate()?,
        Some("upstream.svc".into())
    );

    Ok(())
}

#[test]
fn test_source() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.source().address("10.0.0.1").port(50505);
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.source().address()?, Some("10.0.0.1".to_owned()));
    assert_eq!(stream_info.source().port()?, Some(50505));

    Ok(())
}

#[test]
fn test_destination() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.destination().address("172.0.0.1").port(15000);
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(
        stream_info.destination().address()?,
        Some("172.0.0.1".to_owned())
    );
    assert_eq!(stream_info.destination().port()?, Some(15000));

    Ok(())
}

#[test]
fn test_listener() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.listener()
            .traffic_direction(TrafficDirection::OUTBOUND);
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(
        stream_info.listener().traffic_direction()?,
        Some(TrafficDirection::OUTBOUND)
    );

    Ok(())
}

#[test]
fn test_route() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.route().name("my_route");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.route().name()?, Some("my_route".to_owned()));

    Ok(())
}

#[test]
fn test_cluster() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.cluster().name("my_cluster");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.cluster().name()?, Some("my_cluster".to_owned()));

    Ok(())
}

#[test]
fn test_plugin() -> Result<()> {
    let fake_info = FakeStreamInfo::new().with(|info| {
        info.plugin()
            .name("my_plugin")
            .root_id("my_root_id")
            .vm_id("my_vm_id");
    });
    let stream_info: &dyn StreamInfo = &fake_info;

    assert_eq!(stream_info.plugin().name()?, Some("my_plugin".to_owned()));
    assert_eq!(
        stream_info.plugin().root_id()?,
        Some("my_root_id".to_owned())
    );
    assert_eq!(stream_info.plugin().vm_id()?, Some("my_vm_id".to_owned()));

    Ok(())
}

