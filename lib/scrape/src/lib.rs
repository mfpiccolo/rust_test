#![feature(libc)]
#![feature(cstr_to_str)]
#![feature(cstr_memory)]
#![feature(rustc_private)]
#![feature(str_escape)]
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
use std::iter::repeat;

use hyper::Client;
use hyper::header::Connection;
use std::io::Read;

fn get_page(url: &str) -> String {
    let mut client = Client::new();
    let mut res = client.get(url)
       // set a header
       .header(Connection::close())
       // let 'er go!
       .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    body
}

use tendril::{StrTendril, SliceExt};

#[no_mangle]
pub extern fn parse_page(url: *const libc::c_char) {
  let url_cstr = unsafe { CStr::from_ptr(url) };  // &std::ffi::c_str::CStr
  let url_and_str = url_cstr.to_str().unwrap();  // &str

  let body = get_page(url_and_str);

  let body_tendril = body.to_tendril();
  let body_tendril = body_tendril.try_reinterpret().unwrap();

  let dom: RcDom = parse(one_input(body_tendril), Default::default());

  walk(0, dom.document);
  // let c_body = CString::new(body).unwrap();  // std::ffi::c_str::CString

  // c_body.into_ptr()
}

fn walk(indent: usize, handle: Handle) {
    let node = handle.borrow();
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.node {
        Document
            => println!("#Document"),

        Doctype(ref name, ref public, ref system)
            => println!("<!DOCTYPE {} \"{}\" \"{}\">", *name, *public, *system),

        Text(ref text)
            => println!("#text: {}", text.escape_default()),

        Comment(ref text)
            => println!("<!-- {} -->", text.escape_default()),

        Element(ref name, ref attrs) => {
            // assert!(name.ns == ns!(html));
            print!("<{}", name.local);
            for attr in attrs.iter() {
                // assert!(attr.name.ns == ns!(""));
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        }
    }

    for child in node.children.iter() {
        walk(indent+4, child.clone());
    }
}
