#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
#![feature(rustc_private)]
#![feature(str_escape)]
#![feature(append)]
extern crate libc;

extern crate url;
extern crate hyper;
extern crate html5ever;
extern crate serialize;
extern crate html5ever_dom_sink;
extern crate fnv;

#[macro_use]
extern crate string_cache;
extern crate tendril;

use std::ffi::{CStr,CString};
use tendril::{ByteTendril, ReadExt};
use html5ever::{parse, one_input};
use html5ever_dom_sink::common::{Document, Doctype, Text, Comment, Element};
use html5ever_dom_sink::rcdom::{RcDom, Handle};
use std::iter::repeat;

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
    fn from_vec<T>(mut vec: Vec<T>) -> Array {
        // Important to make length and capacity match
        // A better solution is to track both length and capacity
        // vec.shrink_to_fit();

        let array = Array { data: vec.as_ptr() as *const libc::c_void, len: vec.len() as libc::size_t };

        // Whee! Leak the memory, and now the raw pointer (and
        // eventually C) is the owner.
        // mem::forget(vec);

        array
    }
}

fn get_page(url: &str) -> String {
    let client = Client::new();


    let mut temp_res = client.get(url)
       // set a header
       .header(Connection::close())
       // let 'er go!
       .send();

    let mut res = temp_res.unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    body
}

use tendril::{StrTendril, SliceExt};

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
      Element(ref name, ref attrs) => {
        for attr in attrs.iter() {
            if attr.name.local.to_string() == "href".to_string() {
              urls.push(CString::new(attr.value.to_string()).unwrap().into_ptr());
              // print!("{}\n", attr.value);
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
    // FIXME: don't allocate
    // print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.node {
        Document => {},
            // => println!("#Document"),

        Doctype(ref name, ref public, ref system) => {},
            // => println!("<!DOCTYPE {} \"{}\" \"{}\">", *name, *public, *system),

        Text(ref text) => {},
            // => println!("#text: {}", text.escape_default()),

        Comment(ref text) => {},
            // => println!("<!-- {} -->", text.escape_default()),

        Element(ref name, ref attrs) => {
            if name.local.to_string() == tag_name {
              nodes.push(handle.clone());
            }
        }
    }

    for child in node.children.iter() {
        nodes.append(&mut query(child.clone(), tag_name));
    }
    nodes
}
