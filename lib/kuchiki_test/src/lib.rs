#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]

extern crate libc;
extern crate url;
extern crate hyper;
extern crate kuchiki;

use kuchiki::Html;
use kuchiki::tree::NodeData::Element;
use std::ffi::{CStr,CString};
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

#[no_mangle]
pub extern fn get_links(c_url: *const libc::c_char, c_selector: *const libc::c_char) -> Array {
  let url = ruby_string_to_ref_str(c_url);
  let selector = ruby_string_to_ref_str(c_selector);

  let body = get_page(url);

  let document = Html::from_string(body).parse();
  let links = document.select(selector).unwrap().collect::<Vec<_>>();

  let mut urls: Vec<*const libc::c_char> = vec![];

  for link in links {
    match link.as_node().data {
      Element(ref elem) => {
        for attr in elem.attributes.borrow().iter() {
          let name = attr.0.local.to_string();
          let val  = attr.1;

          if name == "href".to_string() {
            urls.push(CString::new(val.to_string()).unwrap().into_ptr());
          }
        }
      },
      _ => {}
    }
  }

  Array::from_vec(urls)
}

fn ruby_string_to_ref_str<'a>(r_string: *const libc::c_char) -> &'a str {
  unsafe { CStr::from_ptr(r_string) }.to_str().unwrap()
}

