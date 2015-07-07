#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
#![feature(plugin)]

extern crate libc;
extern crate url;
extern crate hyper;
extern crate kuchiki;
extern crate regex;

use kuchiki::Html;
use kuchiki::tree::NodeData::Element;
use std::ffi::{CStr,CString};
use hyper::Client;
use hyper::header::Connection;
use std::io::Read;
use std::mem;
use regex::Regex;

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

/// Call this method from Ruby to get an array of values of text mathes that are inside of nodes that for a given selector on the given page.
/// mathes = Rust.get_text_matches('http://doc.rust-lang.org/regex/regex/index.html', 'p', '(\w*::\w*:?:?\w*:?:?\w*)').to_a
/// => ["Regex::replace", "Regex::with_size_limit"]
#[no_mangle]
pub extern fn get_text_matches(
    c_url: *const libc::c_char,
    c_selector: *const libc::c_char,
    regex_string: *const libc::c_char
  ) -> Array {
  let url = ruby_string_to_ref_str(c_url);
  let selector = ruby_string_to_ref_str(c_selector);
  let body = get_page(url);
  let contains_text = ruby_string_to_ref_str(regex_string);
  let re = Regex::new(contains_text).unwrap();

  let document = Html::from_string(body).parse();
  let elements = document.select(selector).unwrap().collect::<Vec<_>>();

  let mut urls: Vec<*const libc::c_char> = vec![];

  for e in elements {
    for text in e.as_node().text_iter() {
      for cap in re.captures_iter(&text.clone().into_inner()) {
        urls.push(CString::new(cap.at(1).unwrap_or("").to_string()).unwrap().into_ptr());
      }
    }
  }
  Array::from_vec(urls)
}

/// Call this method from Ruby to get an array of values of attributes that match the selctor on the given page.
/// attrs = Rust.get_attrs('http://doc.rust-lang.org/regex/regex/index.html', 'h1#usage', 'class').to_a
/// => ["section-header"]
#[no_mangle]
pub extern fn get_attrs(c_url: *const libc::c_char,
    c_selector: *const libc::c_char,
    c_attr: *const libc::c_char) -> Array {
  let url      = ruby_string_to_ref_str(c_url);
  let selector = ruby_string_to_ref_str(c_selector);
  let html_attr     = ruby_string_to_ref_str(c_attr);

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

          if name == html_attr.to_string() {
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

