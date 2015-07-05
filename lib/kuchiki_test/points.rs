#[no_mangle]
pub extern "C" fn make_point(x: i32, y: i32) -> Box<T> {
    Box::new(5)
}
