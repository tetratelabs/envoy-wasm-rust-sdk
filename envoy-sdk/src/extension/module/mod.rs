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

#[macro_export]
macro_rules! start {
    // Apparently, Rust toolchain doesn't handle well exported name `_start`
    // when a package is compiled to targets other than `wasm32-unknown-unknown`.
    // Specifically, linking issues have been observed with targets `wasm32-wasi`
    // and `x86_64-unknown-linux-gnu`, which blocks unit testing.
    // Therefore, only use export name `_start` when in the context of target
    // `wasm32-unknown-unknown`.

    ($($t:tt)*) => {
        #[cfg_attr(
            all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown"
            ),
            export_name = "_start"
        )]
        #[no_mangle]
        extern "C" fn start() {
            $($t)*
        }
    };
}
