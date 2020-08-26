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

//! Simulates the exact behaviour of `Envoy` or `Proxy Wasm` inside `Envoy`.

use std::cmp;

use envoy::error::format_err;
use envoy::host::{self, ByteString};

/// Reads buffer similarly to `Proxy Wasm` inside Envoy.
pub fn get_buffer_bytes(buf: &[u8], offset: usize, max_size: usize) -> host::Result<ByteString> {
    // implementation based on `proxy-wasm/proxy-wasm-cpp-host`

    // Check for overflow.
    if let (_, true) = offset.overflowing_add(max_size) {
        return Err(format_err!("Status::BadArgument"));
    }
    let max_size = cmp::min(max_size, buf.len() - offset);
    if max_size > 0 {
        return Ok(buf[offset..offset + max_size].to_owned().into());
    }
    Ok(ByteString::default())
}

/// Mutates buffer similarly to `Proxy Wasm` inside Envoy.
pub fn set_buffer_bytes(
    buf: &mut Vec<u8>,
    start: usize,
    length: usize,
    data: &[u8],
) -> host::Result<()> {
    // implementation based on `envoyproxy/envoy-wasm`

    if start == 0 {
        if length == 0 {
            let tail: Vec<u8> = buf.drain(..).collect();
            buf.extend(data);
            buf.extend(tail);
            Ok(())
        } else if length >= buf.len() {
            buf.truncate(0);
            buf.extend(data);
            Ok(())
        } else {
            Err(format_err!("Status::BadArgument"))
        }
    } else if start >= buf.len() {
        buf.extend(data);
        Ok(())
    } else {
        Err(format_err!("Status::BadArgument"))
    }
}
