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

use crate::abi::proxy_wasm::types::Bytes;

use crate::host;

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
/// let plugin_name = stream_info.stream_property(vec!["plugin_name"])?;
///
/// stream_info.set_stream_property(vec!["my_extension", "output"], Some(b"property value"))?;
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
    fn stream_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;

    /// Saves a value in the enclosing context.
    ///
    /// The value will be accessible to other filters on that HTTP stream or TCP connection.
    ///
    /// # Arguments
    ///
    /// * `path`  - property path as an array of path segments
    /// * `value` - an opaque blob of bytes
    fn set_stream_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()>;
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

mod impls {
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::Bytes;

    use super::StreamInfo;
    use crate::host;

    pub(super) struct Host;

    impl StreamInfo for Host {
        fn stream_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
            hostcalls::get_property(path)
        }

        fn set_stream_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()> {
            hostcalls::set_property(path, value)
        }
    }
}
