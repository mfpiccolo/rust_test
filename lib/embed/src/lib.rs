#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
#![feature(rustc_private)]
extern crate libc;

extern crate url;
extern crate hyper;
extern crate html5ever;
extern crate serialize;
extern crate html5ever_dom_sink;

#[macro_use]
extern crate string_cache;
extern crate tendril;
use std::ffi::{CStr,CString};

use tendril::{ByteTendril, ReadExt};
use html5ever::{parse, one_input};
use html5ever_dom_sink::common::{Document, Doctype, Text, Comment, Element};
use html5ever_dom_sink::rcdom::{RcDom, Handle};

use hyper::Client;
use hyper::header::Connection;
use std::io::Read;


#[no_mangle]
pub extern fn get_page(url: *const libc::c_char) -> *const libc::c_char {
    let url_cstr = unsafe { CStr::from_ptr(url) };  // &std::ffi::c_str::CStr
    let url_and_str = url_cstr.to_str().unwrap();  // &str
    let mut client = Client::new();

    let mut res = client.get(url_and_str)
       // set a header
       .header(Connection::close())
       // let 'er go!
       .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let c_body = CString::new(body).unwrap();  // std::ffi::c_str::CString

    c_body.into_ptr()
}
