#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
extern crate libc;

pub struct PlusOneNumbers {
    a: i32,
    b: i32,
}

#[no_mangle]
pub extern fn print_number(x: i32) -> i32 {
    println!("x is: {}", x.to_string());
    x
}

#[no_mangle]
pub extern fn add_numbers(x: i32, y: i32) -> i32 {
    println!("sum is: {}", (x + y).to_string());
    x + y
}

#[no_mangle]
pub extern fn add_struct_vals(pon: PlusOneNumbers) -> i32 {
    pon.a + pon.b
}

#[no_mangle]
pub extern fn return_struct(pon: PlusOneNumbers) -> PlusOneNumbers {
    pon
}

use std::ffi::{CStr,CString};
#[no_mangle]
pub extern fn return_string(test_str: &CStr) -> &CStr {
    test_str
}

#[no_mangle]
pub extern fn print_string_length(mystr: *const libc::c_char) {
    unsafe {
        let slice = CStr::from_ptr(mystr);
        let s_length = slice.to_bytes().len();

        println!("string length: {}", s_length);
    }
}

#[no_mangle]
pub extern fn reverse(s: *const libc::c_char) -> *const libc::c_char {
    let s = unsafe { CStr::from_ptr(s) };
    let s2 = s.to_str().unwrap();
    let s3: String = s2.chars().rev().collect();
    let s4 = CString::new(s3).unwrap();
    s4.into_ptr()
}

#[no_mangle]
pub extern fn concat(s1: *const libc::c_char, s2: *const libc::c_char) -> *const libc::c_char {
    let s1_cstr = unsafe { CStr::from_ptr(s1) };  // &std::ffi::c_str::CStr
    let s2_cstr = unsafe { CStr::from_ptr(s2) };  // &std::ffi::c_str::CStr
    let s1_and_str = s1_cstr.to_str().unwrap();  // &str
    let s2_and_str = s2_cstr.to_str().unwrap();  // &str

    let mut s1_string = s1_and_str.to_string();  // collections::string::String

    s1_string.push_str(s2_and_str);
    // s1_string + s2_and_str); // same thing

    let concated_string = CString::new(s1_string).unwrap();  // std::ffi::c_str::CString

    concated_string.into_ptr() // const i8
}
