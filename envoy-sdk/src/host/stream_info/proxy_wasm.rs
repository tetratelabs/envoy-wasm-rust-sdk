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

//! `Proxy Wasm` format for encoding property values.

use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
use std::time::SystemTime;

use super::types::{ResponseFlags, TrafficDirection};
use crate::host;

use self::types::*;

pub(super) mod types {
    /// A string that is not guaranteed to be UTF-8 encoded.
    pub struct ByteString;
    /// Opaque blob of bytes.
    pub struct _Bytes;
    /// Little endian encoded i64 value.
    pub struct Int64;
    /// Little endian encoded u64 value.
    pub struct UInt64;
    pub struct _Float64;
    /// 1 byte.
    pub struct Bool;
    /// Nanos represented by (i64) rather than (i64, i32).
    pub struct Duration;
    /// UNIX nanos represented by (i64) rather than (i64, i32).
    pub struct Timestamp;
    pub struct _ProtoMessage;
    pub struct _ProtoMap;
    pub struct _ProtoList;
}

/// Represents an encoded property value.
pub(super) struct Value<T> {
    bytes: Vec<u8>,
    _type: PhantomData<T>,
}

impl<T> Value<T> {
    pub fn new(bytes: Vec<u8>) -> Self {
        Value {
            bytes,
            _type: PhantomData,
        }
    }
}

impl TryFrom<Value<ByteString>> for String {
    type Error = host::Error;

    fn try_from(value: Value<ByteString>) -> host::Result<Self> {
        Ok(String::from_utf8(value.bytes)?)
    }
}

impl TryFrom<Value<ByteString>> for host::ByteString {
    type Error = host::Error;

    fn try_from(value: Value<ByteString>) -> host::Result<Self> {
        Ok(value.bytes.into())
    }
}

impl TryFrom<Value<Int64>> for i32 {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let value: i64 = value.try_into()?;
        Ok(value.try_into()?)
    }
}

impl TryFrom<Value<Int64>> for i64 {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let bytes: [u8; std::mem::size_of::<Self>()] = value.bytes.as_slice().try_into()?;
        Ok(Self::from_le_bytes(bytes))
    }
}

impl TryFrom<Value<Int64>> for u64 {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let bytes: [u8; std::mem::size_of::<Self>()] = value.bytes.as_slice().try_into()?;
        Ok(Self::from_le_bytes(bytes))
    }
}

impl TryFrom<Value<Bool>> for bool {
    type Error = host::Error;

    fn try_from(value: Value<Bool>) -> host::Result<Self> {
        let bytes: [u8; 1] = value.bytes.as_slice().try_into()?;
        let value = u8::from_le_bytes(bytes);
        Ok(value != 0)
    }
}

impl TryFrom<Value<Int64>> for u16 {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let value: i64 = value.try_into()?;
        Ok(value.try_into()?)
    }
}

impl TryFrom<Value<Int64>> for u32 {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let value: i64 = value.try_into()?;
        Ok(value.try_into()?)
    }
}

impl TryFrom<Value<UInt64>> for u64 {
    type Error = host::Error;

    fn try_from(value: Value<UInt64>) -> host::Result<Self> {
        let bytes: [u8; std::mem::size_of::<Self>()] = value.bytes.as_slice().try_into()?;
        Ok(Self::from_le_bytes(bytes))
    }
}

impl TryFrom<Value<Int64>> for ResponseFlags {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let value: u64 = value.try_into()?;
        Ok(ResponseFlags::from_bits(value).unwrap_or_else(ResponseFlags::empty))
    }
}

impl TryFrom<Value<Timestamp>> for i64 {
    type Error = host::Error;

    fn try_from(value: Value<Timestamp>) -> host::Result<Self> {
        let bytes: [u8; std::mem::size_of::<Self>()] = value.bytes.as_slice().try_into()?;
        Ok(Self::from_le_bytes(bytes))
    }
}

impl TryFrom<Value<Timestamp>> for SystemTime {
    type Error = host::Error;

    fn try_from(value: Value<Timestamp>) -> host::Result<Self> {
        let nanos: i64 = value.try_into()?;
        let dur = std::time::Duration::from_nanos(nanos.abs() as u64);
        if nanos > 0 {
            Ok(SystemTime::UNIX_EPOCH + dur)
        } else {
            Ok(SystemTime::UNIX_EPOCH - dur)
        }
    }
}

impl TryFrom<Value<Duration>> for i64 {
    type Error = host::Error;

    fn try_from(value: Value<Duration>) -> host::Result<Self> {
        let bytes: [u8; std::mem::size_of::<Self>()] = value.bytes.as_slice().try_into()?;
        Ok(Self::from_le_bytes(bytes))
    }
}

impl TryFrom<Value<Duration>> for std::time::Duration {
    type Error = host::Error;

    fn try_from(value: Value<Duration>) -> host::Result<Self> {
        let nanos: i64 = value.try_into()?;
        let dur = Self::from_nanos(nanos.abs() as u64);
        if nanos > 0 {
            Ok(dur)
        } else {
            Ok(Self::default())
        }
    }
}

impl TryFrom<Value<Int64>> for TrafficDirection {
    type Error = host::Error;

    fn try_from(value: Value<Int64>) -> host::Result<Self> {
        let value: i32 = value.try_into()?;
        Ok(match value {
            0 => TrafficDirection::UNSPECIFIED,
            1 => TrafficDirection::INBOUND,
            2 => TrafficDirection::OUTBOUND,
            _ => TrafficDirection::UNSPECIFIED,
        })
    }
}
