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

//! `Stream Info` properties.

use std::marker::PhantomData;
use std::time::{Duration, SystemTime};

use super::proxy_wasm;
use super::types::{ResponseFlags, TrafficDirection};
use crate::host::ByteString;

/// Represents a property path.
struct Path<'a> {
    inner: PathKind<'a>,
}

impl<'a> Path<'a> {
    pub fn as_ref(&'a self) -> &'a [&'a str] {
        self.inner.as_ref()
    }
}

/// Represents a property path.
enum PathKind<'a> {
    Static(&'static [&'static str]),
    Custom(Vec<&'a str>),
}

impl<'a> PathKind<'a> {
    pub fn as_ref(&'a self) -> &'a [&'a str] {
        use PathKind::*;
        match self {
            Static(ref slice) => slice,
            Custom(ref vec) => &vec,
        }
    }
}

/// Represents an individual property of a stream, e.g.
/// request id, response status code, upstream address, etc.
pub(super) struct Property<'a, T, W> {
    path: Path<'a>,
    type_: PhantomData<T>,
    proxy_wasm_type: PhantomData<W>,
}

impl<'a, T, W> Property<'a, T, W> {
    pub fn path(&'a self) -> &'a [&'a str] {
        self.path.as_ref()
    }
}

/// Enumerates `request` properties.
pub(super) struct Request {}

impl Request {
    /// Request header by name.
    pub fn header(name: &str) -> Property<'_, ByteString, proxy_wasm::types::ByteString> {
        Property {
            path: Path {
                inner: PathKind::Custom(vec!["request", "headers", name]),
            },
            type_: PhantomData::<ByteString>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        }
    }

    /// Request ID.
    pub const ID: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["request", "id"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// Time of the first byte received.
    pub const TIME: &'static Property<'static, SystemTime, proxy_wasm::types::Timestamp> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "time"]),
            },
            type_: PhantomData::<SystemTime>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::Timestamp>,
        };

    /// Total duration of the request.
    pub const DURATION: &'static Property<'static, Duration, proxy_wasm::types::Duration> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "duration"]),
            },
            type_: PhantomData::<Duration>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::Duration>,
        };

    /// Size of the request body.
    pub const SIZE: &'static Property<'static, u64, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["request", "size"]),
        },
        type_: PhantomData::<u64>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// Total size of the request including the headers.
    pub const TOTAL_SIZE: &'static Property<'static, u64, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["request", "total_size"]),
        },
        type_: PhantomData::<u64>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// Request protocol e.g. "HTTP/2".
    pub const PROTOCOL: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "protocol"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The path portion of the URL.
    pub const PATH: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["request", "path"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The path portion of the URL without the query string.
    pub const URL_PATH: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "url_path"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The host portion of the URL.
    pub const HOST: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["request", "host"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// Request method.
    pub const METHOD: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "method"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The scheme portion of the URL.
    pub const SCHEME: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "scheme"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// Referer request header.
    pub const REFERER: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "referer"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// User agent request header.
    pub const USER_AGENT: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["request", "user_agent"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };
}

/// Enumerates `response` properties.
pub(super) struct Response {}

impl Response {
    /// Response header by name.
    pub fn header(name: &str) -> Property<'_, ByteString, proxy_wasm::types::ByteString> {
        Property {
            path: Path {
                inner: PathKind::Custom(vec!["response", "headers", name]),
            },
            type_: PhantomData::<ByteString>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        }
    }

    /// Response trailer by name.
    pub fn trailer(name: &str) -> Property<'_, ByteString, proxy_wasm::types::ByteString> {
        Property {
            path: Path {
                inner: PathKind::Custom(vec!["response", "trailers", name]),
            },
            type_: PhantomData::<ByteString>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        }
    }

    /// Response HTTP status code.
    pub const STATUS_CODE: &'static Property<'static, u16, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["response", "code"]),
        },
        type_: PhantomData::<u16>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// Size of the response body.
    pub const SIZE: &'static Property<'static, u64, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["response", "size"]),
        },
        type_: PhantomData::<u64>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// Total size of the response including the approximate uncompressed size of the headers and the trailers.
    pub const TOTAL_SIZE: &'static Property<'static, u64, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["response", "total_size"]),
        },
        type_: PhantomData::<u64>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// Additional details about the response beyond the standard response code.
    pub const FLAGS: &'static Property<'static, ResponseFlags, proxy_wasm::types::Int64> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["response", "flags"]),
            },
            type_: PhantomData::<ResponseFlags>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
        };

    /// Response gRPC status code.
    pub const GRPC_STATUS: &'static Property<'static, i32, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["response", "grpc_status"]),
        },
        type_: PhantomData::<i32>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };
}

/// Enumerates `connection` properties.
pub(super) struct Connection {}

impl Connection {
    /// Connection ID.
    pub const ID: &'static Property<'static, u64, proxy_wasm::types::UInt64> = &Property {
        path: Path {
            inner: PathKind::Static(&["connection_id"]),
        },
        type_: PhantomData::<u64>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::UInt64>,
    };

    /// Indicates whether TLS is applied to the downstream connection and the peer ceritificate is presented.
    pub const IS_MTLS: &'static Property<'static, bool, proxy_wasm::types::Bool> = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "mtls"]),
        },
        type_: PhantomData::<bool>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Bool>,
    };

    /// Requested server name in the downstream TLS connection.
    pub const REQUESTED_SERVER_NAME: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "requested_server_name"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// TLS version of the downstream TLS connection.
    pub const TLS_VERSION: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["connection", "tls_version"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The subject field of the local certificate in the downstream TLS connection.
    pub const SUBJECT_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "subject_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The subject field of the peer certificate in the downstream TLS connection.
    pub const SUBJECT_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "subject_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first URI entry in the SAN field of the local certificate in the downstream TLS connection.
    pub const URI_SAN_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "uri_san_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first URI entry in the SAN field of the peer certificate in the downstream TLS connection.
    pub const URI_SAN_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "uri_san_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first DNS entry in the SAN field of the local certificate in the downstream TLS connection.
    pub const DNS_SAN_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "dns_san_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first DNS entry in the SAN field of the peer certificate in the downstream TLS connection.
    pub const DNS_SAN_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["connection", "dns_san_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };
}

/// Enumerates `upstream` properties.
pub(super) struct Upstream {}

impl Upstream {
    /// Upstream connection remote address.
    pub const ADDRESS: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["upstream", "address"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// Upstream connection remote port.
    pub const PORT: &'static Property<'static, u32, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "port"]),
        },
        type_: PhantomData::<u32>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };

    /// The local address of the upstream connection.
    pub const LOCAL_ADDRESS: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["upstream", "local_address"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The upstream transport failure reason e.g. certificate validation failed.
    pub const TRANSPORT_FAILURE_REASON: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "transport_failure_reason"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// TLS version of the upstream TLS connection.
    pub const TLS_VERSION: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["upstream", "tls_version"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// The subject field of the local certificate in the upstream TLS connection.
    pub const SUBJECT_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "subject_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The subject field of the peer certificate in the upstream TLS connection.
    pub const SUBJECT_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "subject_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first URI entry in the SAN field of the local certificate in the upstream TLS connection.
    pub const URI_SAN_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "uri_san_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first URI entry in the SAN field of the peer certificate in the upstream TLS connection.
    pub const URI_SAN_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "uri_san_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first DNS entry in the SAN field of the local certificate in the upstream TLS connection.
    pub const DNS_SAN_LOCAL_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "dns_san_local_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// The first DNS entry in the SAN field of the peer certificate in the upstream TLS connection.
    pub const DNS_SAN_PEER_CERTIFICATE: &'static Property<
        'static,
        String,
        proxy_wasm::types::ByteString,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["upstream", "dns_san_peer_certificate"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };
}

/// Enumerates `source` properties.
pub(super) struct Source {}

impl Source {
    /// Downstream connection remote address.
    pub const ADDRESS: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["source", "address"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// Downstream connection remote port.
    pub const PORT: &'static Property<'static, u32, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["source", "port"]),
        },
        type_: PhantomData::<u32>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };
}

/// Enumerates `destination` properties.
pub(super) struct Destination {}

impl Destination {
    /// Downstream connection local address.
    pub const ADDRESS: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["destination", "address"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// Downstream connection local port.
    pub const PORT: &'static Property<'static, u32, proxy_wasm::types::Int64> = &Property {
        path: Path {
            inner: PathKind::Static(&["destination", "port"]),
        },
        type_: PhantomData::<u32>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };
}

/// Enumerates `plugin` properties.
pub(super) struct Plugin {}

impl Plugin {
    /// Plugin name.
    pub const NAME: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["plugin_name"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };

    /// Plugin Root ID.
    pub const ROOT_ID: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["plugin_root_id"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };

    /// Plugin VM ID.
    pub const VM_ID: &'static Property<'static, String, proxy_wasm::types::ByteString> =
        &Property {
            path: Path {
                inner: PathKind::Static(&["plugin_vm_id"]),
            },
            type_: PhantomData::<String>,
            proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
        };
}

/// Enumerates `listener` properties.
pub(super) struct Listener {}

impl Listener {
    /// Traffic direction.
    pub const TRAFFIC_DIRECTION: &'static Property<
        'static,
        TrafficDirection,
        proxy_wasm::types::Int64,
    > = &Property {
        path: Path {
            inner: PathKind::Static(&["listener_direction"]),
        },
        type_: PhantomData::<TrafficDirection>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::Int64>,
    };
}

/// Enumerates `cluster` properties.
pub(super) struct Cluster {}

impl Cluster {
    /// Cluster name.
    pub const NAME: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["cluster_name"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };
}

/// Enumerates `route` properties.
pub(super) struct Route {}

impl Route {
    /// Route name.
    pub const NAME: &'static Property<'static, String, proxy_wasm::types::ByteString> = &Property {
        path: Path {
            inner: PathKind::Static(&["route_name"]),
        },
        type_: PhantomData::<String>,
        proxy_wasm_type: PhantomData::<proxy_wasm::types::ByteString>,
    };
}
