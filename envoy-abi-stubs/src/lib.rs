#[no_mangle]
pub extern "C" fn proxy_log(_: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_get_current_time_nanoseconds(_: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_set_effective_context(_: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_done() -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_set_tick_period_milliseconds(_: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_get_configuration(_: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_get_buffer_bytes(_: i32, _: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_get_header_map_pairs(_: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_continue_request() -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_continue_response() -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_send_local_response(
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_clear_route_cache() -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_get_property(_: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_set_property(_: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_http_call(
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
) -> i32 {
    unimplemented!()
}


#[no_mangle]
pub extern "C" fn proxy_get_shared_data(_: i32, _: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_set_shared_data(_: i32, _: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_register_shared_queue(_: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_resolve_shared_queue(_: i32, _: i32, _: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_dequeue_shared_queue(_: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn proxy_enqueue_shared_queue(_: i32, _: i32, _: i32) -> i32 {
    unimplemented!()
}
