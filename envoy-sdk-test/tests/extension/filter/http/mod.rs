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

use envoy::extension::filter::http::{self, FilterDataStatus, FilterHeadersStatus};
use envoy::extension::{self, ExtensionFactory, HttpFilter, InstanceId};
use envoy::host::buffer::Transform;
use envoy::host::{HeaderMap, Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeEnvoy;

use self::noop::NoOpHttpFilterFactory;

mod noop;

#[test]
fn test_request_response() -> Result<()> {
    struct TestFilter<'a> {
        stats: &'a dyn Stats,
    }
    impl HttpFilter for TestFilter<'_> {
        fn on_request_headers(
            &mut self,
            _num_headers: usize,
            _ops: &dyn http::RequestHeadersOps,
        ) -> Result<FilterHeadersStatus> {
            self.stats.counter("test_filter.requests_total")?.inc()?;
            Ok(FilterHeadersStatus::Continue)
        }

        fn on_request_body(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn http::RequestBodyOps,
        ) -> Result<FilterDataStatus> {
            if data_size > 0 {
                let mut data = ops.request_data(0, data_size)?.into_vec();
                if !data.is_empty() {
                    data.remove(0);
                }
                ops.mutate_request_data(Transform::replace_with(&data))?;
            }
            Ok(FilterDataStatus::Continue)
        }

        fn on_response_body(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn http::ResponseBodyOps,
        ) -> extension::Result<FilterDataStatus> {
            if data_size > 0 {
                let mut data = ops.response_data(0, data_size)?.into_vec();
                data.extend("!".bytes());
                ops.mutate_response_data(Transform::replace_with(&data))?;
            }
            Ok(FilterDataStatus::Continue)
        }
    }

    struct TestFilterFactory<'a> {
        stats: &'a dyn Stats,
    }
    impl<'a> ExtensionFactory for TestFilterFactory<'a> {
        type Extension = TestFilter<'a>;

        fn name() -> &'static str {
            "test"
        }

        fn new_extension(
            &mut self,
            _instance_id: InstanceId,
        ) -> extension::Result<Self::Extension> {
            Ok(TestFilter { stats: self.stats })
        }
    }
    impl<'a> TestFilterFactory<'a> {
        fn new(stats: &'a dyn Stats) -> Self {
            TestFilterFactory { stats }
        }
    }

    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(TestFilterFactory::new(&fake.stats))
        .configure("{}")?;

    let mut stream = fake_listener.new_http_stream()?;
    {
        let request_headers = HeaderMap::builder()
            .header(":authority", "example.org")
            .header(":method", "GET")
            .header(":path", "/stuff")
            .build();

        let status = stream.simulate_headers_from_downstream(&request_headers, false)?;

        assert_eq!(status, http::FilterHeadersStatus::Continue);
        assert_eq!(
            fake.stats.counter("test_filter.requests_total")?.value()?,
            1
        );
        assert_eq!(stream.upstream().received_headers(), Some(&request_headers));
        assert_eq!(stream.upstream().has_received_end(), false);

        let status = stream.simulate_data_from_downstream("hello", false)?;

        assert_eq!(status, http::FilterDataStatus::Continue);
        assert_eq!(stream.upstream().drain_received_body(), b"ello");
        assert_eq!(stream.upstream().has_received_end(), false);

        let status = stream.simulate_data_from_downstream("world", false)?;

        assert_eq!(status, http::FilterDataStatus::Continue);
        assert_eq!(stream.upstream().drain_received_body(), b"orld");
        assert_eq!(stream.upstream().has_received_end(), false);

        let request_trailers = HeaderMap::builder()
            .header("my-trailer", "my-value")
            .build();

        let status = stream.simulate_trailers_from_downstream(&request_trailers)?;

        assert_eq!(status, http::FilterTrailersStatus::Continue);
        assert_eq!(
            stream.upstream().received_trailers(),
            Some(&request_trailers)
        );
        assert_eq!(stream.upstream().has_received_end(), true);

        let response_headers = HeaderMap::builder().header(":status", "200").build();

        let status = stream.simulate_headers_from_upstream(&response_headers, false)?;

        assert_eq!(status, http::FilterHeadersStatus::Continue);
        assert_eq!(
            stream.downstream().received_headers(),
            Some(&response_headers)
        );
        assert_eq!(stream.downstream().has_received_end(), false);

        let status = stream.simulate_data_from_upstream("hi", false)?;

        assert_eq!(status, http::FilterDataStatus::Continue);
        assert_eq!(stream.downstream().drain_received_body(), b"hi!");
        assert_eq!(stream.downstream().has_received_end(), false);

        let status = stream.simulate_data_from_upstream("there", false)?;

        assert_eq!(status, http::FilterDataStatus::Continue);
        assert_eq!(stream.downstream().drain_received_body(), b"there!");
        assert_eq!(stream.downstream().has_received_end(), false);

        let response_trailers = HeaderMap::builder()
            .header("grpc-status", "0")
            .header("grpc-message", "OK")
            .build();

        let status = stream.simulate_trailers_from_upstream(&response_trailers)?;

        assert_eq!(status, http::FilterTrailersStatus::Continue);
        assert_eq!(
            stream.downstream().received_trailers(),
            Some(&response_trailers)
        );
        assert_eq!(stream.downstream().has_received_end(), true);
    }

    let mut stream2 = fake_listener.new_http_stream()?;
    {
        let request_headers = HeaderMap::builder()
            .header(":authority", "example.org")
            .header(":method", "HEAD")
            .header(":path", "/stuff")
            .build();

        let status = stream2.simulate_headers_from_downstream(&request_headers, true)?;
        assert_eq!(status, http::FilterHeadersStatus::Continue);
        assert_eq!(
            fake.stats.counter("test_filter.requests_total")?.value()?,
            2
        );
    }

    Ok(())
}

#[test]
fn test_local_reply_on_request_headers() -> Result<()> {
    struct TestFilter;
    impl HttpFilter for TestFilter {
        fn on_request_headers(
            &mut self,
            _num_headers: usize,
            ops: &dyn http::RequestHeadersOps,
        ) -> Result<FilterHeadersStatus> {
            ops.send_response(401, &[("header", b"value")], Some(b"body"))?;
            Ok(FilterHeadersStatus::StopIteration)
        }
    }

    struct TestFilterFactory;
    impl ExtensionFactory for TestFilterFactory {
        type Extension = TestFilter;

        fn name() -> &'static str {
            "test"
        }

        fn new_extension(
            &mut self,
            _instance_id: InstanceId,
        ) -> extension::Result<Self::Extension> {
            Ok(TestFilter)
        }
    }

    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(TestFilterFactory)
        .configure("{}")?;

    let mut stream = fake_listener.new_http_stream()?;
    {
        let request_headers = HeaderMap::builder()
            .header(":authority", "example.org")
            .header(":method", "GET")
            .header(":path", "/stuff")
            .build();

        let status = stream.simulate_headers_from_downstream(request_headers, false)?;

        assert_eq!(status, http::FilterHeadersStatus::StopIteration);
        assert_eq!(stream.upstream().has_received_headers(), false);

        assert_eq!(
            stream.downstream().received_headers(),
            Some(
                &HeaderMap::builder()
                    .header("header", "value")
                    .header(":status", "401")
                    .build()
            )
        );
        assert_eq!(stream.downstream().received_body(), b"body");
        assert_eq!(stream.downstream().received_trailers(), None);
        assert_eq!(stream.downstream().has_received_end(), true);
    }

    Ok(())
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot send request headers to Envoy for the second time"
)]
fn test_downstream_sends_headers_second_time() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, false)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);

    stream
        .simulate_headers_from_downstream(&request_headers, false)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot keep sending request data to Envoy after signaling end of stream"
)]
fn test_downstream_sends_data_after_end_of_stream() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);

    stream
        .simulate_data_from_downstream("hello", false)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot send request trailers to Envoy after signaling end of stream"
)]
fn test_downstream_sends_trailers_after_end_of_stream() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);

    let request_trailers = HeaderMap::builder()
        .header("my-trailer", "my-value")
        .build();

    stream
        .simulate_trailers_from_downstream(&request_trailers)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot send request data to Envoy prior to sending request headers first"
)]
fn test_downstream_starts_from_request_body() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    stream
        .simulate_data_from_downstream("hello", false)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot send request trailers to Envoy prior to sending request headers first"
)]
fn test_downstream_starts_from_request_trailers() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_trailers = HeaderMap::builder()
        .header("my-trailer", "my-value")
        .build();

    stream
        .simulate_trailers_from_downstream(&request_trailers)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first"
)]
fn test_upstream_sends_headers_prior_to_receiving_request() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let response_headers = HeaderMap::builder().header(":status", "200").build();

    stream
        .simulate_headers_from_upstream(&response_headers, false)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first"
)]
fn test_upstream_sends_data_prior_to_receiving_request() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    stream.simulate_data_from_upstream("hi", false).unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving request headers first"
)]
fn test_upstream_sends_trailers_prior_to_receiving_request() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let response_trailers = HeaderMap::builder()
        .header("grpc-status", "0")
        .header("grpc-message", "OK")
        .build();

    stream
        .simulate_trailers_from_upstream(&response_trailers)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot send request headers to Envoy for the second time"
)]
fn test_upstream_sends_headers_second_time() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, false)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.upstream().has_received_headers(), true);

    let response_headers = HeaderMap::builder().header(":status", "200").build();

    let status = stream
        .simulate_headers_from_upstream(&response_headers, false)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);

    stream
        .simulate_headers_from_upstream(&response_headers, false)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot send response data to Envoy prior to sending response headers first"
)]
fn test_upstream_starts_from_response_body() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, false)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.upstream().has_received_headers(), true);

    stream.simulate_data_from_upstream("hi", false).unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot send response trailers to Envoy prior to sending response headers first"
)]
fn test_upstream_starts_from_response_trailers() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, false)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.upstream().has_received_headers(), true);

    let response_trailers = HeaderMap::builder()
        .header("grpc-status", "0")
        .header("grpc-message", "OK")
        .build();

    stream
        .simulate_trailers_from_upstream(&response_trailers)
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot keep sending response data to Envoy after signaling end of stream"
)]
fn test_upstream_sends_data_after_end_of_stream() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.upstream().has_received_headers(), true);

    let response_headers = HeaderMap::builder().header(":status", "200").build();

    let status = stream
        .simulate_headers_from_upstream(&response_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.downstream().has_received_headers(), true);
    assert_eq!(stream.downstream().has_received_end(), true);

    stream.simulate_data_from_upstream("hi", false).unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot send response trailers to Envoy after signaling end of stream"
)]
fn test_upstream_sends_trailers_after_end_of_stream() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .filter(NoOpHttpFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut stream = fake_listener.new_http_stream().unwrap();

    let request_headers = HeaderMap::builder()
        .header(":authority", "example.org")
        .header(":method", "GET")
        .header(":path", "/stuff")
        .build();

    let status = stream
        .simulate_headers_from_downstream(&request_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.upstream().has_received_headers(), true);

    let response_headers = HeaderMap::builder().header(":status", "200").build();

    let status = stream
        .simulate_headers_from_upstream(&response_headers, true)
        .unwrap();
    assert_eq!(status, http::FilterHeadersStatus::Continue);
    assert_eq!(stream.downstream().has_received_headers(), true);
    assert_eq!(stream.downstream().has_received_end(), true);

    let response_trailers = HeaderMap::builder()
        .header("grpc-status", "0")
        .header("grpc-message", "OK")
        .build();

    stream
        .simulate_trailers_from_upstream(&response_trailers)
        .unwrap();
}
