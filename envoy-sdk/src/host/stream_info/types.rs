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

//! Auxiliary `Stream Info` types.

use std::fmt;

use bitflags::bitflags;

bitflags! {
    /// Response flags.
    pub struct ResponseFlags : u64 {
        /// Local server healthcheck failed.
        const FAILED_LOCAL_HEALTH_CHECK = 0x1;
        /// No healthy upstream.
        const NO_HEALTHY_UPSTREAM = 0x2;
        /// Request timeout on upstream.
        const UPSTREAM_REQUEST_TIMEOUT = 0x4;
        /// Local codec level reset was sent on the stream.
        const LOCAL_RESET = 0x8;
        /// Remote codec level reset was received on the stream.
        const UPSTREAM_REMOTE_RESET = 0x10;
        /// Local reset by a connection pool due to an initial connection failure.
        const UPSTREAM_CONNECTION_FAILURE = 0x20;
        /// If the stream was locally reset due to connection termination.
        const UPSTREAM_CONNECTION_TERMINATION = 0x40;
        /// The stream was reset because of a resource overflow.
        const UPSTREAM_OVERFLOW = 0x80;
        /// No route found for a given request.
        const NO_ROUTE_FOUND = 0x100;
        /// Request was delayed before proxying.
        const DELAY_INJECTED = 0x200;
        /// Abort with error code was injected.
        const FAULT_INJECTED = 0x400;
        /// Request was ratelimited locally by rate limit filter.
        const RATE_LIMITED = 0x800;
        /// Request was unauthorized by external authorization service.
        const UNAUTHORIZED_EXTERNAL_SERVICE = 0x1000;
        /// Unable to call Ratelimit service.
        const RATE_LIMIT_SERVICE_ERROR = 0x2000;
        /// If the stream was reset due to a downstream connection termination.
        const DOWNSTREAM_CONNECTION_TERMINATION = 0x4000;
        /// Exceeded upstream retry limit.
        const UPSTREAM_RETRY_LIMIT_EXCEEDED = 0x8000;
        /// Request hit the stream idle timeout;  triggering a 408.
        const STREAM_IDLE_TIMEOUT = 0x10000;
        /// Request specified x-envoy-* header values that failed strict header checks.
        const INVALID_ENVOY_REQUEST_HEADERS = 0x20000;
        /// Downstream request had an HTTP protocol error
        const DOWNSTREAM_PROTOCOL_ERROR = 0x40000;
        /// Upstream request reached to user defined max stream duration.
        const UPSTREAM_MAX_STREAM_DURATION_REACHED = 0x80000;
        /// True if the response was served from an Envoy cache filter.
        const RESPONSE_FROM_CACHE_FILTER = 0x100000;
        /// Filter config was not received within the permitted warming deadline.
        const NO_FILTER_CONFIG_FOUND = 0x200000;
    }
}

impl Default for ResponseFlags {
    fn default() -> Self {
        ResponseFlags::empty()
    }
}

impl fmt::Display for ResponseFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut flags = vec![];
        if self.contains(ResponseFlags::FAILED_LOCAL_HEALTH_CHECK) {
            flags.push("LH");
        }
        if self.contains(ResponseFlags::NO_HEALTHY_UPSTREAM) {
            flags.push("UH");
        }
        if self.contains(ResponseFlags::UPSTREAM_REQUEST_TIMEOUT) {
            flags.push("UT");
        }
        if self.contains(ResponseFlags::LOCAL_RESET) {
            flags.push("LR");
        }
        if self.contains(ResponseFlags::UPSTREAM_REMOTE_RESET) {
            flags.push("UR");
        }
        if self.contains(ResponseFlags::UPSTREAM_CONNECTION_FAILURE) {
            flags.push("UF");
        }
        if self.contains(ResponseFlags::UPSTREAM_CONNECTION_TERMINATION) {
            flags.push("UC");
        }
        if self.contains(ResponseFlags::UPSTREAM_OVERFLOW) {
            flags.push("UO");
        }
        if self.contains(ResponseFlags::NO_ROUTE_FOUND) {
            flags.push("NR");
        }
        if self.contains(ResponseFlags::DELAY_INJECTED) {
            flags.push("DI");
        }
        if self.contains(ResponseFlags::FAULT_INJECTED) {
            flags.push("FI");
        }
        if self.contains(ResponseFlags::RATE_LIMITED) {
            flags.push("RL");
        }
        if self.contains(ResponseFlags::FAULT_INJECTED) {
            flags.push("FI");
        }
        if self.contains(ResponseFlags::UNAUTHORIZED_EXTERNAL_SERVICE) {
            flags.push("UAEX");
        }
        if self.contains(ResponseFlags::RATE_LIMIT_SERVICE_ERROR) {
            flags.push("RL");
        }
        if self.contains(ResponseFlags::DOWNSTREAM_CONNECTION_TERMINATION) {
            flags.push("DC");
        }
        if self.contains(ResponseFlags::UPSTREAM_RETRY_LIMIT_EXCEEDED) {
            flags.push("URX");
        }
        if self.contains(ResponseFlags::STREAM_IDLE_TIMEOUT) {
            flags.push("SI");
        }
        if self.contains(ResponseFlags::INVALID_ENVOY_REQUEST_HEADERS) {
            flags.push("IH");
        }
        if self.contains(ResponseFlags::DOWNSTREAM_PROTOCOL_ERROR) {
            flags.push("DPE");
        }
        if self.contains(ResponseFlags::UPSTREAM_MAX_STREAM_DURATION_REACHED) {
            flags.push("UMSDR");
        }
        if self.contains(ResponseFlags::RESPONSE_FROM_CACHE_FILTER) {
            flags.push("RFCF");
        }
        if self.contains(ResponseFlags::NO_FILTER_CONFIG_FOUND) {
            flags.push("NFCF");
        }
        write!(f, "{}", flags.join(","))
    }
}

/// Identifies the direction of the traffic relative to the local Envoy.
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum TrafficDirection {
    /// Default option is unspecified.
    UNSPECIFIED = 0,
    /// The transport is used for incoming traffic.
    INBOUND = 1,
    /// The transport is used for outgoing traffic.
    OUTBOUND = 2,
}

impl Default for TrafficDirection {
    fn default() -> Self {
        TrafficDirection::UNSPECIFIED
    }
}
