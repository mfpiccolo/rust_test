#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
#![feature(rustc_private)]
#![feature(append)]

extern crate libc;
extern crate url;
extern crate hyper;
extern crate html5ever;
extern crate serialize;
extern crate html5ever_dom_sink;
extern crate fnv;
extern crate tendril;
#[macro_use]
extern crate string_cache;

use std::ffi::{CStr,CString};
use html5ever::{parse, one_input};
use html5ever_dom_sink::common::Element;
use html5ever_dom_sink::rcdom::{RcDom, Handle};
use hyper::Client;
use hyper::header::Connection;
use std::io::Read;
use std::mem;
use libc::size_t;

#[repr(C)]
pub struct Array {
  len: libc::size_t,
  data: *const libc::c_void,
}

impl Array {
  fn from_vec<T>(vec: Vec<T>) -> Array {
    let array = Array { data: vec.as_ptr() as *const libc::c_void, len: vec.len() as libc::size_t };
    mem::forget(vec);
    array
  }
}

fn get_page(url: &str) -> String {
  let client = Client::new();

  let temp_res = client.get(url).header(Connection::close()).send();

  let mut res = temp_res.unwrap();

  let mut body = String::new();
  res.read_to_string(&mut body).unwrap();

  body
}

use tendril::SliceExt;

#[no_mangle]
pub extern fn get_links(url: *const libc::c_char) -> Array {
  let url_cstr = unsafe { CStr::from_ptr(url) };  // &std::ffi::c_str::CStr
  let url_and_str = url_cstr.to_str().unwrap();  // &str

  let body = get_page(url_and_str);

  let body_tendril = body.to_tendril();
  let body_tendril = body_tendril.try_reinterpret().unwrap();

  let dom: RcDom = parse(one_input(body_tendril), Default::default());

  let links = query(dom.document, "a");

  let mut urls: Vec<*const libc::c_char> = vec![];

  for link in links {
    match link.borrow().node {
      Element(_, ref attrs) => {
        for attr in attrs.iter() {
          if attr.name.local.to_string() == "href".to_string() {
            urls.push(CString::new(attr.value.to_string()).unwrap().into_ptr());
          }
        };
      },
      _ => {}
    }
  }

  Array::from_vec(urls)
}

fn query(handle: Handle, tag_name: &str) -> Vec<Handle> {
  let mut nodes: Vec<Handle> = Vec::new();
  let node = handle.borrow();

  match node.node {
    Element(ref name, _) => {
      if name.local.to_string() == tag_name {
        nodes.push(handle.clone());
      }
    },
    _ => {}
  }

  for child in node.children.iter() {
    nodes.append(&mut query(child.clone(), tag_name));
  }
  nodes
}
