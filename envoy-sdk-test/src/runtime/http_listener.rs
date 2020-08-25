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

//! Fake `Envoy` environment for `HTTP`-level extensions.

use std::cell::{Ref, RefCell, RefMut};

use envoy::extension::filter::http;
use envoy::extension::{self, factory, ExtensionFactory, HttpFilter};
use envoy::host::buffer::{internal::TransformExecutor, Transform};
use envoy::host::{self, ByteString, HeaderMap};

use super::envoy_mime;
use super::{FakeEnvoy, FakeListenerBuilder};
use crate::extension::filter::http::DynHttpFilterFactory;

/// Factory of fake `Envoy` `Listeners` for testing `HTTP`-level extensions.
pub struct FakeHttpListenerBuilder<'a> {
    listener: FakeListenerBuilder<'a>,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn HttpFilter + 'a>> + 'a>>,
}

/// Fake `Envoy` `Listener` for testing `Http Filter` extensions.
pub struct FakeHttpListener<'a> {
    envoy: &'a FakeEnvoy,
    filter_factory: Box<dyn ExtensionFactory<Extension = Box<dyn HttpFilter + 'a>> + 'a>,
}

/// Fake `Envoy` `HTTP Stream` for testing `Http Filter` extensions.
pub struct FakeHttpStream<'a> {
    filter: RefCell<Box<dyn HttpFilter + 'a>>,
    state: RefCell<FakeHttpStreamState>,
}

/// Encapsulates state of a fake `TCP` connection.
#[derive(Debug, Default)]
struct FakeHttpStreamState {
    request_started: bool,
    request_ended: bool,
    request_headers: HeaderMap,
    // this buffer is only valid for the duration of `on_request_body` callback
    request_data: Vec<u8>,
    request_buffered_data: Vec<u8>,
    request_trailers: HeaderMap,

    response_started: bool,
    response_ended: bool,
    response_headers: HeaderMap,
    // this buffer is only valid for the duration of `on_response_body` callback
    response_data: Vec<u8>,
    response_buffered_data: Vec<u8>,
    response_trailers: HeaderMap,

    request_flow: Flow,
    response_flow: Flow,

    /// Assumed state of the `Downstream`.
    downstream: FakeHttpDownstream,
    /// Assumed state of the `Upstream`.
    upstream: FakeHttpUpstream,
}

/// Envoy concept borrowed "as is".
#[derive(Debug, Clone, Copy)]
enum IterationState {
    Continue,            // Iteration has not stopped for any frame type.
    StopSingleIteration, // Iteration has stopped for headers, 100-continue, or data.
    _StopAllBuffer,      // Iteration has stopped for all frame types, and following data should
    // be buffered.
    _StopAllWatermark, // Iteration has stopped for all frame types, and following data should
                       // be buffered until high watermark is reached.
}

impl Default for IterationState {
    fn default() -> Self {
        IterationState::Continue
    }
}

#[derive(Default, Debug)]
struct Flow {
    iteration_state: IterationState,
    headers: Option<http::FilterHeadersStatus>,
    body: Option<http::FilterDataStatus>,
    trailers: Option<http::FilterTrailersStatus>,
}

/// Encapsulates state of the implicit `Envoy -> Upstream` interactions.
#[derive(Debug, Default)]
pub struct FakeHttpDownstream {
    received_headers: Option<HeaderMap>,
    received_body: Vec<u8>,
    received_trailers: Option<HeaderMap>,
    received_end: bool,
}

/// Encapsulates state of the implicit `Downstream <- Envoy` interactions.
#[derive(Debug, Default)]
pub struct FakeHttpUpstream {
    received_headers: Option<HeaderMap>,
    received_body: Vec<u8>,
    received_trailers: Option<HeaderMap>,
    received_end: bool,
}

impl<'a> FakeHttpListenerBuilder<'a> {
    pub(super) fn new(listener: FakeListenerBuilder<'a>) -> Self {
        FakeHttpListenerBuilder {
            listener,
            filter_factory: None,
        }
    }

    /// Adds a given `HttpFilter` extension to the fake `Envoy` `Listener`.
    pub fn filter<T>(mut self, filter_factory: T) -> Self
    where
        T: ExtensionFactory + 'a,
        T::Extension: HttpFilter,
    {
        self.filter_factory = Some(Box::new(DynHttpFilterFactory::wrap(filter_factory)));
        self
    }

    /// Finishes building a fake `Envoy` `Listener` by applying a given configuration to
    /// the `HttpFilter` extension.
    pub fn configure<C>(self, config: C) -> extension::Result<FakeHttpListener<'a>>
    where
        C: AsRef<[u8]>,
    {
        let mut filter_factory = self.filter_factory.expect(
            "HTTP Filter extension factory must be added prior to calling `configure(...)`",
        );
        filter_factory.on_configure(config.as_ref().into(), &NoOps)?;
        Ok(FakeHttpListener {
            envoy: self.listener.envoy,
            filter_factory,
        })
    }
}

impl<'a> FakeHttpListener<'a> {
    /// Returns a new `HTTP Stream`.
    pub fn new_http_stream(&mut self) -> extension::Result<FakeHttpStream<'a>> {
        let filter = self
            .filter_factory
            .new_extension(self.envoy.generator.new_instance_id())?;
        Ok(FakeHttpStream {
            filter: RefCell::new(filter),
            state: RefCell::new(FakeHttpStreamState::default()),
        })
    }
}

impl FakeHttpDownstream {
    /// Simulates `Downstream <- Envoy` response headers.
    fn receive_headers(&mut self, headers: HeaderMap, end_of_stream: bool) {
        assert_eq!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send response headers to the downstream for the second time");
        self.received_end |= end_of_stream;
        self.received_headers = Some(headers);
    }

    /// Simulates `Downstream <- Envoy` response data.
    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_end, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the downstream after ending HTTP stream");
        assert_ne!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send response data to the downstream prior to sending response headers first");
        self.received_end |= end_of_stream;
        self.received_body.extend(data);
    }

    /// Simulates `Downstream <- Envoy` response trailers.
    fn receive_trailers(&mut self, trailers: HeaderMap, end_of_stream: bool) {
        assert_eq!(self.received_trailers, None, "unit test is trying to do something that actual Envoy would never do: don't send response trailers to the downstream for the second time");
        assert_eq!(self.received_end, false, "unit test is trying to do something that actual Envoy would never do: don't send response trailers to the downstream after ending HTTP stream");
        assert_ne!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send response trailers to the downstream prior to sending response headers first");
        self.received_end |= end_of_stream;
        self.received_trailers = Some(trailers);
    }

    /// Returns response headers received from `Envoy`.
    pub fn received_headers(&self) -> Option<&HeaderMap> {
        self.received_headers.as_ref()
    }

    /// Peeks into the response data received from `Envoy`.
    pub fn received_body(&self) -> &[u8] {
        self.received_body.as_ref()
    }

    /// Returns response data received from `Envoy` since the last call to this method.
    pub fn drain_received_body(&mut self) -> Vec<u8> {
        self.received_body.drain(..).collect()
    }

    /// Returns response trailers received from `Envoy`.
    pub fn received_trailers(&self) -> Option<&HeaderMap> {
        self.received_trailers.as_ref()
    }

    /// Returns `true` after `Downstream` has received response headers from `Envoy`.
    pub fn has_received_headers(&self) -> bool {
        self.received_headers.is_some()
    }

    /// Returns `true` after `Downstream` has received end of stream from `Envoy`.
    pub fn has_received_end(&self) -> bool {
        self.received_end
    }
}

impl FakeHttpUpstream {
    /// Simulates `Envoy -> Upstream` request headers.
    fn receive_headers(&mut self, headers: HeaderMap, end_of_stream: bool) {
        assert_eq!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send request headers to the upstream for the second time");
        self.received_end |= end_of_stream;
        self.received_headers = Some(headers);
    }

    /// Simulates `Envoy -> Upstream` request data.
    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_end, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the upstream after ending HTTP stream");
        assert_ne!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send request data to the upstream prior to sending request headers first");
        self.received_end |= end_of_stream;
        self.received_body.extend(data);
    }

    /// Simulates `Envoy -> Upstream` request trailers.
    fn receive_trailers(&mut self, trailers: HeaderMap, end_of_stream: bool) {
        assert_eq!(self.received_trailers, None, "unit test is trying to do something that actual Envoy would never do: don't send request trailers to the upstream for the second time");
        assert_eq!(self.received_end, false, "unit test is trying to do something that actual Envoy would never do: don't send request trailers to the upstream after ending HTTP stream");
        assert_ne!(self.received_headers, None, "unit test is trying to do something that actual Envoy would never do: don't send request trailers to the upstream prior to sending request headers first");
        self.received_end |= end_of_stream;
        self.received_trailers = Some(trailers);
    }

    /// Returns request headers received from `Envoy`.
    pub fn received_headers(&self) -> Option<&HeaderMap> {
        self.received_headers.as_ref()
    }

    /// Peeks into the request data received from `Envoy`.
    pub fn received_body(&self) -> &[u8] {
        self.received_body.as_ref()
    }

    /// Returns request data received from `Envoy` since the last call to this method.
    pub fn drain_received_body(&mut self) -> Vec<u8> {
        self.received_body.drain(..).collect()
    }

    /// Returns `true` after `Downstream <- Envoy` stream has ended.
    pub fn has_received_headers(&self) -> bool {
        self.received_headers.is_some()
    }

    /// Returns request trailers received from `Envoy`.
    pub fn received_trailers(&self) -> Option<&HeaderMap> {
        self.received_trailers.as_ref()
    }

    /// Returns `true` after `Envoy -> Upstream` stream has ended.
    pub fn has_received_end(&self) -> bool {
        self.received_end
    }
}

impl<'a> FakeHttpStream<'a> {
    fn request_data(&self) -> Ref<'_, Vec<u8>> {
        let state = self.state.borrow();
        // simulate behaviour of `envoyproxy/envoy-wasm`:
        //  1) if `on_request_body` returned `StopIterationAndBuffer` last time, use `request_buffered_data`
        //  2) otherwise, use data passed as an argument to `on_request_body`
        match state.request_flow.body {
            Some(http::FilterDataStatus::StopIterationAndBuffer) => {
                Ref::map(state, |v| &v.request_buffered_data)
            }
            _ => Ref::map(state, |v| &v.request_data),
        }
    }

    fn request_data_mut(&self) -> RefMut<'_, Vec<u8>> {
        let state = self.state.borrow_mut();
        // simulate behaviour of `envoyproxy/envoy-wasm`:
        //  1) if `on_request_body` returned `StopIterationAndBuffer` last time, use `request_buffered_data`
        //  2) otherwise, use data passed as an argument to `on_request_body`
        match state.request_flow.body {
            Some(http::FilterDataStatus::StopIterationAndBuffer) => {
                RefMut::map(state, |v| &mut v.request_buffered_data)
            }
            _ => RefMut::map(state, |v| &mut v.request_data),
        }
    }

    fn response_data(&self) -> Ref<'_, Vec<u8>> {
        let state = self.state.borrow();
        // simulate behaviour of `envoyproxy/envoy-wasm`:
        //  1) if `on_response_body` returned `StopIterationAndBuffer` last time, use `response_buffered_data`
        //  2) otherwise, use data passed as an argument to `on_response_body`
        match state.response_flow.body {
            Some(http::FilterDataStatus::StopIterationAndBuffer) => {
                Ref::map(state, |v| &v.response_buffered_data)
            }
            _ => Ref::map(state, |v| &v.response_data),
        }
    }

    fn response_data_mut(&self) -> RefMut<'_, Vec<u8>> {
        let state = self.state.borrow_mut();
        // simulate behaviour of `envoyproxy/envoy-wasm`:
        //  1) if `on_response_body` returned `StopIterationAndBuffer` last time, use `response_buffered_data`
        //  2) otherwise, use data passed as an argument to `on_response_body`
        match state.response_flow.body {
            Some(http::FilterDataStatus::StopIterationAndBuffer) => {
                RefMut::map(state, |v| &mut v.response_buffered_data)
            }
            _ => RefMut::map(state, |v| &mut v.response_data),
        }
    }

    pub fn simulate_continue_request_headers() -> extension::Result<http::FilterHeadersStatus> {
        Ok(http::FilterHeadersStatus::Continue)
    }

    /// Simulate `Downstream -> Envoy` request headers.
    pub fn simulate_headers_from_downstream<H>(
        &mut self,
        headers: H,
        end_of_stream: bool,
    ) -> extension::Result<http::FilterHeadersStatus>
    where
        H: Into<HeaderMap>,
    {
        assert_eq!(self.state.borrow().request_started, false, "unit test is trying to do something that actual Envoy would never do: downstream cannot send request headers to Envoy for the second time");
        self.state.borrow_mut().request_started = true;
        self.state.borrow_mut().request_headers = headers.into();
        self.state.borrow_mut().request_ended |= end_of_stream;

        let num_headers = self.state.borrow().request_headers.len();
        let status = self
            .filter
            .borrow_mut()
            .on_request_headers(num_headers, self);

        match status {
            Ok(http::FilterHeadersStatus::Continue) => {
                self.state.borrow_mut().request_flow.iteration_state = IterationState::Continue;
                let headers = self.state.borrow().request_headers.clone();
                self.state
                    .borrow_mut()
                    .upstream
                    .receive_headers(headers, end_of_stream);
            }
            Ok(http::FilterHeadersStatus::StopIteration) => {
                self.state.borrow_mut().request_flow.iteration_state =
                    IterationState::StopSingleIteration;
            }
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        if let Ok(status) = status {
            self.state.borrow_mut().request_flow.headers = Some(status);
        }
        status
    }

    /// Simulate `Downstream -> Envoy` request data.
    pub fn simulate_data_from_downstream<B>(
        &mut self,
        data: B,
        end_of_stream: bool,
    ) -> extension::Result<http::FilterDataStatus>
    where
        B: AsRef<[u8]>,
    {
        assert_eq!(self.state.borrow().request_ended, false, "unit test is trying to do something that actual Envoy would never do: downstream cannot keep sending request data to Envoy after signaling end of stream");
        assert_eq!(self.state.borrow().request_started, true, "unit test is trying to do something that actual Envoy would never do: downstream cannot send request data to Envoy prior to sending request headers first");
        self.state.borrow_mut().request_ended |= end_of_stream;
        self.state.borrow_mut().request_data = data.as_ref().to_vec();

        let buf_len = self.request_data().len();
        let status = self
            .filter
            .borrow_mut()
            .on_request_body(buf_len, end_of_stream, self);

        match status {
            Ok(http::FilterDataStatus::Continue) => {
                let iteration_state = self.state.borrow().request_flow.iteration_state;
                match iteration_state {
                    IterationState::Continue => {
                        let data = self.state.borrow_mut().request_data.drain(..).collect();
                        self.state
                            .borrow_mut()
                            .upstream
                            .receive_data(data, end_of_stream);
                    }
                    IterationState::StopSingleIteration => {
                        self.state.borrow_mut().request_buffered_data.extend(
                            self.state
                                .borrow_mut()
                                .request_data
                                .drain(..)
                                .collect::<Vec<u8>>(),
                        );
                        // TODO: continue
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(http::FilterDataStatus::StopIterationAndBuffer) => {
                // simulate Envoy who puts given data into buffer
                //TODO:
                // if self.state.borrow().request_flow.body_stopped {
                //     self.state
                //     .borrow_mut()
                //     .request_buffered_data
                //     .extend(data.as_ref());
                // }
                // self.state.borrow_mut().request_flow.body_stopped = true;
            }
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Downstream -> Envoy` request trailers.
    pub fn simulate_trailers_from_downstream<H>(
        &mut self,
        trailers: H,
    ) -> extension::Result<http::FilterTrailersStatus>
    where
        H: Into<HeaderMap>,
    {
        assert_eq!(self.state.borrow().request_ended, false, "unit test is trying to do something that actual Envoy would never do: downstream cannot send request trailers to Envoy after signaling end of stream");
        assert_eq!(self.state.borrow().request_started, true, "unit test is trying to do something that actual Envoy would never do: downstream cannot send request trailers to Envoy prior to sending request headers first");
        self.state.borrow_mut().request_trailers = trailers.into();
        self.state.borrow_mut().request_ended |= true;

        let num_trailers = self.state.borrow().request_trailers.len();
        let status = self
            .filter
            .borrow_mut()
            .on_request_trailers(num_trailers, self);

        match status {
            Ok(http::FilterTrailersStatus::Continue) => {
                let trailers = self.state.borrow().request_trailers.clone();
                let end_of_stream = self.state.borrow_mut().request_ended;
                self.state
                    .borrow_mut()
                    .upstream
                    .receive_trailers(trailers, end_of_stream);
            }
            Ok(http::FilterTrailersStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Envoy <- Upstream` response headers.
    pub fn simulate_headers_from_upstream<H>(
        &mut self,
        headers: H,
        end_of_stream: bool,
    ) -> extension::Result<http::FilterHeadersStatus>
    where
        H: Into<HeaderMap>,
    {
        assert_eq!(self.state.borrow().upstream.has_received_headers(), true, "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first");
        assert_eq!(self.state.borrow().response_started, false, "unit test is trying to do something that actual Envoy would never do: upstream cannot send request headers to Envoy for the second time");
        self.state.borrow_mut().response_started = true;
        self.state.borrow_mut().response_headers = headers.into();
        self.state.borrow_mut().response_ended |= end_of_stream;

        let num_headers = self.state.borrow().response_headers.len();
        let status = self
            .filter
            .borrow_mut()
            .on_response_headers(num_headers, self);

        match status {
            Ok(http::FilterHeadersStatus::Continue) => {
                let headers = self.state.borrow().response_headers.clone();
                self.state
                    .borrow_mut()
                    .downstream
                    .receive_headers(headers, end_of_stream);
            }
            Ok(http::FilterHeadersStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Envoy <- Upstream` response data.
    pub fn simulate_data_from_upstream<B>(
        &mut self,
        data: B,
        end_of_stream: bool,
    ) -> extension::Result<http::FilterDataStatus>
    where
        B: AsRef<[u8]>,
    {
        assert_eq!(self.state.borrow().upstream.has_received_headers(), true, "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first");
        assert_eq!(self.state.borrow().response_ended, false, "unit test is trying to do something that actual Envoy would never do: upstream cannot keep sending response data to Envoy after signaling end of stream");
        assert_eq!(self.state.borrow().response_started, true, "unit test is trying to do something that actual Envoy would never do: upstream cannot send response data to Envoy prior to sending response headers first");
        self.state
            .borrow_mut()
            .response_buffered_data
            .extend(data.as_ref());
        self.state.borrow_mut().response_ended |= end_of_stream;

        let buf_len = self.state.borrow().response_buffered_data.len();
        let status = self
            .filter
            .borrow_mut()
            .on_response_body(buf_len, end_of_stream, self);

        match status {
            Ok(http::FilterDataStatus::Continue) => {
                let data = self
                    .state
                    .borrow_mut()
                    .response_buffered_data
                    .drain(..)
                    .collect();
                self.state
                    .borrow_mut()
                    .downstream
                    .receive_data(data, end_of_stream);
            }
            Ok(http::FilterDataStatus::StopIterationAndBuffer) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Envoy <- Upstream` response trailers.
    pub fn simulate_trailers_from_upstream<H>(
        &mut self,
        trailers: H,
    ) -> extension::Result<http::FilterTrailersStatus>
    where
        H: Into<HeaderMap>,
    {
        assert_eq!(self.state.borrow().upstream.has_received_headers(), true, "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first");
        assert_eq!(self.state.borrow().response_ended, false, "unit test is trying to do something that actual Envoy would never do: upstream cannot send response trailers to Envoy after signaling end of stream");
        assert_eq!(self.state.borrow().response_started, true, "unit test is trying to do something that actual Envoy would never do: upstream cannot send response trailers to Envoy prior to sending response headers first");
        self.state.borrow_mut().response_trailers = trailers.into();
        self.state.borrow_mut().response_ended |= true;

        let num_trailers = self.state.borrow().response_trailers.len();
        let status = self
            .filter
            .borrow_mut()
            .on_response_trailers(num_trailers, self);

        match status {
            Ok(http::FilterTrailersStatus::Continue) => {
                let trailers = self.state.borrow().response_trailers.clone();
                let end_of_stream = self.state.borrow().response_ended;
                self.state
                    .borrow_mut()
                    .downstream
                    .receive_trailers(trailers, end_of_stream);
            }
            Ok(http::FilterTrailersStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Peeks into the request body buffer.
    pub fn peek_request_buffered_data(&self) -> Vec<u8> {
        self.state.borrow_mut().request_buffered_data.clone()
    }

    /// Returns alleged state of the `Downstream` resulting from implicit `Downstream <- Envoy` interactions.
    pub fn downstream(&mut self) -> RefMut<'_, FakeHttpDownstream> {
        RefMut::map(self.state.borrow_mut(), |s| &mut s.downstream)
    }

    /// Returns alleged state of the `Upstream` resulting from implicit `Envoy -> Upstream` interactions.
    pub fn upstream(&mut self) -> RefMut<'_, FakeHttpUpstream> {
        RefMut::map(self.state.borrow_mut(), |s| &mut s.upstream)
    }
}

impl<'a> http::RequestHeadersOps for FakeHttpStream<'a> {
    fn request_headers(&self) -> host::Result<HeaderMap> {
        Ok(self.state.borrow().request_headers.clone())
    }

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .state
            .borrow()
            .request_headers
            .get(name)
            .map(Clone::clone))
    }

    fn set_request_headers(&self, headers: &HeaderMap) -> host::Result<()> {
        self.state.borrow_mut().request_headers = headers.clone();
        Ok(())
    }

    fn set_request_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        self.state.borrow_mut().request_headers.insert(name, value);
        Ok(())
    }

    fn remove_request_header(&self, name: &str) -> host::Result<()> {
        self.state.borrow_mut().request_headers.remove(name);
        Ok(())
    }
}

impl<'a> http::RequestBodyOps for FakeHttpStream<'a> {
    fn request_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString> {
        envoy_mime::get_buffer_bytes(&self.request_data(), offset, max_size)
    }

    fn mutate_request_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, length: usize, data: &[u8]| {
            envoy_mime::set_buffer_bytes(&mut self.request_data_mut(), start, length, data)
        })
    }
}

impl<'a> http::RequestTrailersOps for FakeHttpStream<'a> {
    fn request_trailers(&self) -> host::Result<HeaderMap> {
        Ok(self.state.borrow().request_trailers.clone())
    }

    fn request_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .state
            .borrow()
            .request_trailers
            .get(name)
            .map(Clone::clone))
    }

    fn set_request_trailers(&self, headers: &HeaderMap) -> host::Result<()> {
        self.state.borrow_mut().request_trailers = headers.clone();
        Ok(())
    }

    fn set_request_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        self.state.borrow_mut().request_trailers.insert(name, value);
        Ok(())
    }

    fn remove_request_trailer(&self, name: &str) -> host::Result<()> {
        self.state.borrow_mut().request_trailers.remove(name);
        Ok(())
    }
}

impl<'a> http::RequestFlowOps for FakeHttpStream<'a> {
    fn resume_request(&self) -> host::Result<()> {
        Ok(()) // TODO
    }

    fn clear_route_cache(&self) -> host::Result<()> {
        // TODO(yskopets): implement
        Ok(())
    }

    fn send_response(
        &self,
        status_code: u32,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> host::Result<()> {
        assert_eq!(self.state.borrow().downstream.has_received_headers(), false, "unit test is trying to do something that actual Envoy would never do: it is not possible to send local reply once response headers have already been sent to downstream");
        let mut headers: HeaderMap = headers.iter().collect();
        headers.insert(":status", status_code.to_string());

        self.state.borrow_mut().response_headers = headers.clone();
        self.state.borrow_mut().response_buffered_data = Vec::new();
        self.state.borrow_mut().response_trailers = HeaderMap::default();

        self.state
            .borrow_mut()
            .downstream
            .receive_headers(headers, body.is_none());
        if let Some(body) = body {
            self.state
                .borrow_mut()
                .downstream
                .receive_data(body.into(), true);
        }
        self.state.borrow_mut().response_ended |= true;
        Ok(())
    }
}

impl<'a> http::ResponseHeadersOps for FakeHttpStream<'a> {
    fn response_headers(&self) -> host::Result<HeaderMap> {
        Ok(self.state.borrow().response_headers.clone())
    }

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .state
            .borrow()
            .response_headers
            .get(name)
            .map(Clone::clone))
    }

    fn set_response_headers(&self, headers: &HeaderMap) -> host::Result<()> {
        self.state.borrow_mut().response_headers = headers.clone();
        Ok(())
    }

    fn set_response_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        self.state.borrow_mut().response_headers.insert(name, value);
        Ok(())
    }

    fn remove_response_header(&self, name: &str) -> host::Result<()> {
        self.state.borrow_mut().response_headers.remove(name);
        Ok(())
    }
}

impl<'a> http::ResponseBodyOps for FakeHttpStream<'a> {
    fn response_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString> {
        envoy_mime::get_buffer_bytes(&self.response_data(), offset, max_size)
    }

    fn mutate_response_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, length: usize, data: &[u8]| {
            envoy_mime::set_buffer_bytes(&mut self.response_data_mut(), start, length, data)
        })
    }
}

impl<'a> http::ResponseTrailersOps for FakeHttpStream<'a> {
    fn response_trailers(&self) -> host::Result<HeaderMap> {
        Ok(self.state.borrow().response_trailers.clone())
    }

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .state
            .borrow()
            .response_trailers
            .get(name)
            .map(Clone::clone))
    }

    fn set_response_trailers(&self, headers: &HeaderMap) -> host::Result<()> {
        self.state.borrow_mut().response_trailers = headers.clone();
        Ok(())
    }

    fn set_response_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        self.state
            .borrow_mut()
            .response_trailers
            .insert(name, value);
        Ok(())
    }

    fn remove_response_trailer(&self, name: &str) -> host::Result<()> {
        self.state.borrow_mut().response_trailers.remove(name);
        Ok(())
    }
}

impl<'a> http::ResponseFlowOps for FakeHttpStream<'a> {
    fn resume_response(&self) -> host::Result<()> {
        Ok(()) // TODO
    }
}

struct NoOps;

impl factory::ConfigureOps for NoOps {}
