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

//! Extensions to [`proxy_wasm::hostcalls`].
//!
//! [`proxy-wasm::hostcalls`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/hostcalls/index.html

use std::time::{Duration, SystemTime};

use proxy_wasm::hostcalls;

use super::types::{
    BufferType, HttpRequestHandle, MapType, MetricHandle, MetricType, OptimisticLockVersion,
    SharedQueueHandle, Status,
};
use crate::error::format_err;
use crate::host::{self, Bytes, HeaderMap, HeaderValue};

// Configuration API

pub fn get_configuration() -> host::Result<Bytes> {
    hostcalls::get_configuration()
        .map(Bytes::from)
        .map_err(|err| format_err!(err))
}

// Lifecycle API

pub fn done() -> host::Result<()> {
    hostcalls::done().map_err(|err| format_err!(err))
}

// Headers/Body manipulation API

pub fn add_map_value(map_type: MapType, key: &str, value: &HeaderValue) -> host::Result<()> {
    // TODO(yskopets): change `proxy-wasm` to represent header value as `&[u8]` rather than `&str`
    hostcalls::add_map_value(map_type, key, unsafe {
        core::str::from_utf8_unchecked(value.as_bytes())
    })
    .map_err(|err| format_err!(err))
}

pub fn get_buffer(buffer_type: BufferType, start: usize, max_size: usize) -> host::Result<Bytes> {
    hostcalls::get_buffer(buffer_type, start, max_size)
        .map(Bytes::from)
        .map_err(|err| format_err!(err))
}

pub fn get_map(map_type: MapType) -> host::Result<HeaderMap> {
    hostcalls::get_map(map_type)
        .map(HeaderMap::from)
        .map_err(|err| format_err!(err))
}

pub fn set_map(map_type: MapType, map: &HeaderMap) -> host::Result<()> {
    let mut headers = Vec::with_capacity(map.len());
    for (name, value) in map {
        // TODO(yskopets): change `proxy-wasm` to represent header value as `&[u8]` rather than `&str`
        headers.push((name.as_ref(), unsafe {
            core::str::from_utf8_unchecked(value.as_bytes())
        }));
    }
    hostcalls::set_map(map_type, headers).map_err(|err| format_err!(err))
}

pub fn get_map_value(map_type: MapType, name: &str) -> host::Result<Option<HeaderValue>> {
    hostcalls::get_map_value(map_type, name)
        .map(|o| o.map(HeaderValue::from))
        .map_err(|err| format_err!(err))
}

pub fn set_map_value(
    map_type: MapType,
    name: &str,
    value: Option<&HeaderValue>,
) -> host::Result<()> {
    // TODO(yskopets): change `proxy-wasm` to represent header value as `&[u8]` rather than `&str`
    hostcalls::set_map_value(
        map_type,
        name,
        value.map(|value| unsafe { core::str::from_utf8_unchecked(value.as_bytes()) }),
    )
    .map_err(|err| format_err!(err))
}

// HTTP Flow API

pub fn clear_http_route_cache() -> host::Result<()> {
    hostcalls::clear_http_route_cache().map_err(|err| format_err!(err))
}

pub fn resume_http_request() -> host::Result<()> {
    hostcalls::resume_http_request().map_err(|err| format_err!(err))
}

pub fn resume_http_response() -> host::Result<()> {
    hostcalls::resume_http_response().map_err(|err| format_err!(err))
}

pub fn send_http_response(
    status_code: u32,
    headers: &[(&str, &[u8])],
    body: Option<&[u8]>,
) -> host::Result<()> {
    let mut unsafe_headers = Vec::with_capacity(headers.len());
    for (name, value) in headers {
        // TODO(yskopets): change `proxy-wasm` to represent header value as `&[u8]` rather than `&str`
        unsafe_headers.push((*name, unsafe { core::str::from_utf8_unchecked(value) }));
    }
    hostcalls::send_http_response(status_code, unsafe_headers, body).map_err(|err| format_err!(err))
}

// Shared Queue

pub fn register_shared_queue(name: &str) -> host::Result<SharedQueueHandle> {
    hostcalls::register_shared_queue(name)
        .map(SharedQueueHandle::from)
        .map_err(|err| format_err!(err))
}

pub fn resolve_shared_queue(vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>> {
    hostcalls::resolve_shared_queue(vm_id, name)
        .map(|o| o.map(SharedQueueHandle::from))
        .map_err(|err| format_err!(err))
}

pub fn dequeue_shared_queue(queue_id: SharedQueueHandle) -> host::Result<Option<Bytes>> {
    hostcalls::dequeue_shared_queue(queue_id.as_id())
        .map(|data| data.map(Bytes::from))
        .map_err(|err| format_err!(err))
}

pub fn enqueue_shared_queue(queue_id: SharedQueueHandle, value: &[u8]) -> host::Result<()> {
    hostcalls::enqueue_shared_queue(
        queue_id.as_id(),
        if value.is_empty() { None } else { Some(value) },
    )
    .map_err(|err| format_err!(err))
}

// Time API

pub fn get_current_time() -> host::Result<SystemTime> {
    hostcalls::get_current_time().map_err(|err| format_err!(err))
}

// HTTP Client API

pub fn dispatch_http_call(
    upstream: &str,
    headers: &[(&str, &[u8])],
    body: Option<&[u8]>,
    trailers: &[(&str, &[u8])],
    timeout: Duration,
) -> host::Result<HttpRequestHandle> {
    // TODO(yskopets): change `proxy-wasm` to represent header value as `&[u8]` rather than `&str`
    let headers = headers
        .iter()
        .map(|(name, value)| (*name, unsafe { core::str::from_utf8_unchecked(value) }))
        .collect();
    let trailers = trailers
        .iter()
        .map(|(name, value)| (*name, unsafe { core::str::from_utf8_unchecked(value) }))
        .collect();
    hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
        .map(HttpRequestHandle::from)
        .map_err(|err| format_err!(err))
}

// Stream Info API

pub fn get_property(path: &[&str]) -> host::Result<Option<Bytes>> {
    // TODO(yskopets): change `proxy-wasm` to accept &[&str] instead of Vec<&str>
    hostcalls::get_property(path.into())
        .map(|data| data.map(Bytes::from))
        .map_err(|err| format_err!(err))
}

pub fn set_property(path: &[&str], value: &[u8]) -> host::Result<()> {
    // TODO(yskopets): change `proxy-wasm` to accept &[&str] instead of Vec<&str>
    hostcalls::set_property(
        path.into(),
        if value.is_empty() { None } else { Some(value) },
    )
    .map_err(|err| format_err!(err))
}

// Shared data API

pub fn get_shared_data(key: &str) -> host::Result<(Option<Bytes>, Option<OptimisticLockVersion>)> {
    hostcalls::get_shared_data(key)
        .map(|(value, version)| (value.map(Bytes::from), version))
        .map_err(|err| format_err!(err))
}

pub fn set_shared_data(
    key: &str,
    value: &[u8],
    version: Option<OptimisticLockVersion>,
) -> host::Result<()> {
    hostcalls::set_shared_data(
        key,
        if value.is_empty() { None } else { Some(value) },
        version,
    )
    .map_err(|err| format_err!(err))
}

// Stats API

extern "C" {
    fn proxy_define_metric(
        metric_type: MetricType,
        metric_name_data: *const u8,
        metric_name_size: usize,
        return_metric_id: *mut u32,
    ) -> Status;
}

pub fn define_metric(metric_type: MetricType, metric_name: &str) -> host::Result<MetricHandle> {
    unsafe {
        let mut return_metric_id: u32 = 0;
        match proxy_define_metric(
            metric_type,
            metric_name.as_ptr(),
            metric_name.len(),
            &mut return_metric_id,
        ) {
            Status::Ok => Ok(MetricHandle::from(return_metric_id)),
            status => Err(host::function("env", "proxy_define_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_increment_metric(metric_id: u32, offset: i64) -> Status;
}

pub fn increment_metric(metric_handle: MetricHandle, offset: i64) -> host::Result<()> {
    unsafe {
        match proxy_increment_metric(metric_handle.as_id(), offset) {
            Status::Ok => Ok(()),
            status => Err(host::function("env", "proxy_increment_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_record_metric(metric_id: u32, value: u64) -> Status;
}

pub fn record_metric(metric_handle: MetricHandle, value: u64) -> host::Result<()> {
    unsafe {
        match proxy_record_metric(metric_handle.as_id(), value) {
            Status::Ok => Ok(()),
            status => Err(host::function("env", "proxy_record_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_get_metric(metric_id: u32, return_metric_value: *mut u64) -> Status;
}

pub fn get_metric(metric_handle: MetricHandle) -> host::Result<u64> {
    unsafe {
        let mut return_metric_value: u64 = 0;
        match proxy_get_metric(metric_handle.as_id(), &mut return_metric_value) {
            Status::Ok => Ok(return_metric_value),
            status => Err(host::function("env", "proxy_increment_metric")
                .into_call_error(status)
                .into()),
        }
    }
}
