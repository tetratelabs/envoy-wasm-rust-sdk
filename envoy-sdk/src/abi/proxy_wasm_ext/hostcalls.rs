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
    BufferType, Bytes, HttpRequestHandle, MapType, MetricHandle, MetricType, SharedQueueHandle,
    Status,
};
use crate::host;

// Configuration API

pub fn get_configuration() -> host::Result<Option<Bytes>> {
    hostcalls::get_configuration().map_err(|status| {
        host::function("env", "proxy_get_configuration")
            .into_call_error(status)
            .into()
    })
}

// Lifecycle API

pub fn done() -> host::Result<()> {
    hostcalls::done().map_err(|status| {
        host::function("env", "proxy_done")
            .into_call_error(status)
            .into()
    })
}

// Headers/Body manipulation API

pub fn get_map(map_type: MapType) -> host::Result<Vec<(String, String)>> {
    hostcalls::get_map(map_type).map_err(|status| {
        host::function("env", "proxy_get_header_map_pairs")
            .into_call_error(status)
            .into()
    })
}

pub fn set_map(map_type: MapType, headers: Vec<(&str, &str)>) -> host::Result<()> {
    hostcalls::set_map(map_type, headers).map_err(|status| {
        host::function("env", "proxy_set_header_map_pairs")
            .into_call_error(status)
            .into()
    })
}

pub fn get_map_value(map_type: MapType, key: &str) -> host::Result<Option<String>> {
    hostcalls::get_map_value(map_type, key).map_err(|status| {
        host::function("env", "proxy_get_header_map_value")
            .into_call_error(status)
            .into()
    })
}

pub fn set_map_value(map_type: MapType, name: &str, value: Option<&str>) -> host::Result<()> {
    hostcalls::set_map_value(map_type, name, value).map_err(|status| match value {
        Some(_) => host::function("env", "proxy_replace_header_map_value")
            .into_call_error(status)
            .into(),
        None => host::function("env", "proxy_remove_header_map_value")
            .into_call_error(status)
            .into(),
    })
}

pub fn add_map_value(map_type: MapType, name: &str, value: &str) -> host::Result<()> {
    hostcalls::add_map_value(map_type, name, value).map_err(|status| {
        host::function("env", "proxy_add_header_map_value")
            .into_call_error(status)
            .into()
    })
}

pub fn get_buffer(
    buffer_type: BufferType,
    start: usize,
    max_size: usize,
) -> host::Result<Option<Bytes>> {
    hostcalls::get_buffer(buffer_type, start, max_size).map_err(|status| {
        host::function("env", "proxy_get_buffer_bytes")
            .into_call_error(status)
            .into()
    })
}

// HTTP Flow API

pub fn resume_http_request() -> host::Result<()> {
    hostcalls::resume_http_request().map_err(|status| {
        host::function("env", "proxy_continue_request")
            .into_call_error(status)
            .into()
    })
}

pub fn send_http_response(
    status_code: u32,
    headers: Vec<(&str, &str)>,
    body: Option<&[u8]>,
) -> host::Result<()> {
    hostcalls::send_http_response(status_code, headers, body).map_err(|status| {
        host::function("env", "proxy_send_local_response")
            .into_call_error(status)
            .into()
    })
}

pub fn clear_http_route_cache() -> host::Result<()> {
    hostcalls::clear_http_route_cache().map_err(|status| {
        host::function("env", "proxy_clear_route_cache")
            .into_call_error(status)
            .into()
    })
}

pub fn resume_http_response() -> host::Result<()> {
    hostcalls::resume_http_response().map_err(|status| {
        host::function("env", "proxy_continue_response")
            .into_call_error(status)
            .into()
    })
}

// Shared Queue

pub fn register_shared_queue(name: &str) -> host::Result<SharedQueueHandle> {
    hostcalls::register_shared_queue(name)
        .map_err(|status| {
            host::function("env", "proxy_register_shared_queue")
                .into_call_error(status)
                .into()
        })
        .map(SharedQueueHandle::from)
}

pub fn resolve_shared_queue(vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>> {
    hostcalls::resolve_shared_queue(vm_id, name)
        .map_err(|status| {
            host::function("env", "proxy_resolve_shared_queue")
                .into_call_error(status)
                .into()
        })
        .map(|o| o.map(SharedQueueHandle::from))
}

pub fn dequeue_shared_queue(queue_id: SharedQueueHandle) -> host::Result<Option<Bytes>> {
    hostcalls::dequeue_shared_queue(queue_id.as_id()).map_err(|status| {
        host::function("env", "proxy_dequeue_shared_queue")
            .into_call_error(status)
            .into()
    })
}

pub fn enqueue_shared_queue(queue_id: SharedQueueHandle, value: Option<&[u8]>) -> host::Result<()> {
    hostcalls::enqueue_shared_queue(queue_id.as_id(), value).map_err(|status| {
        host::function("env", "proxy_enqueue_shared_queue")
            .into_call_error(status)
            .into()
    })
}

// Time API

pub fn get_current_time() -> host::Result<SystemTime> {
    hostcalls::get_current_time().map_err(|status| {
        host::function("env", "proxy_get_current_time_nanoseconds")
            .into_call_error(status)
            .into()
    })
}

// HTTP Client API

pub fn dispatch_http_call(
    upstream: &str,
    headers: Vec<(&str, &str)>,
    body: Option<&[u8]>,
    trailers: Vec<(&str, &str)>,
    timeout: Duration,
) -> host::Result<HttpRequestHandle> {
    hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
        .map_err(|status| {
            host::function("env", "proxy_http_call")
                .into_call_error(status)
                .into()
        })
        .map(HttpRequestHandle::from)
}

// Stream Info API

pub fn get_property(path: Vec<&str>) -> host::Result<Option<Bytes>> {
    hostcalls::get_property(path).map_err(|status| {
        host::function("env", "proxy_get_property")
            .into_call_error(status)
            .into()
    })
}

pub fn set_property(path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()> {
    hostcalls::set_property(path, value).map_err(|status| {
        host::function("env", "proxy_set_property")
            .into_call_error(status)
            .into()
    })
}

// Shared data API

pub fn get_shared_data(key: &str) -> host::Result<(Option<Bytes>, Option<u32>)> {
    hostcalls::get_shared_data(key).map_err(|status| {
        host::function("env", "proxy_get_shared_data")
            .into_call_error(status)
            .into()
    })
}

pub fn set_shared_data(key: &str, value: Option<&[u8]>, cas: Option<u32>) -> host::Result<()> {
    hostcalls::set_shared_data(key, value, cas).map_err(|status| {
        host::function("env", "proxy_set_shared_data")
            .into_call_error(status)
            .into()
    })
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
