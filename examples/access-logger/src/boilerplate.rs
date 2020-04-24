#[cfg(test)]
extern crate envoy_abi_stubs;

#[cfg(feature = "malloc")]
mod support {
    extern crate extension_abi_memory;

    #[no_mangle]
    pub extern "C" fn malloc(size: usize) -> *mut u8 {
        extension_abi_memory::allocate(size)
    }
}
