#![feature(type_alias_enum_variants)]

extern crate cfg_if;
extern crate wasm_bindgen;

#[macro_use]
mod utils;

use std::net::Ipv4Addr;
use std::cmp::min;
use std::fmt::Write;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use utils::{math_log2, set_panic_hook, log};

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[derive(Debug)]
enum Error {
    RIRStatsMalformed(usize)
}

impl From<Error> for JsValue {
    fn from(err: Error) -> JsValue {
        match err {
            Error::RIRStatsMalformed(lineno) => JsValue::from(format!("RIR stats is malformed at line {}.", lineno)),
        }
    }
}

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, wasm-worker!".to_string()
}



#[wasm_bindgen]
pub fn routes_from_rir_stats(raw_data: &str, country: &str) -> Result<String, JsValue> {
    set_panic_hook();
    // https://www.apnic.net/about-apnic/corporate-documents/documents/resource-guidelines/rir-statistics-exchange-format/
    let mut raw_entries: Vec<(u32, u32)> = Vec::new();
    for (index, entry) in raw_data.split("\n").enumerate() {
        if !entry.starts_with("#") && !entry.is_empty() {
            let fields: Vec<&str> = entry.split("|").collect();
            if fields.len() >= 7 && fields[1] == country && fields[2] == "ipv4" {
                let ip: Ipv4Addr = fields[3].parse().map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
                let count: u32 = fields[4].parse().map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
                let [a, b, c, d] = ip.octets();
                let ip = (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32;
                raw_entries.push((ip, count));
            }
        }
    }
    let raw_len = raw_entries.len();
    raw_entries.sort();
    let mut merged_entries = Vec::new();
    let mut last_entry = raw_entries[0];
    for entry in raw_entries.into_iter().skip(1) {
        if entry.0 - last_entry.0 == last_entry.1 {
            last_entry = (last_entry.0, last_entry.1 + entry.1)
        }
        else {
            merged_entries.push(last_entry);
            last_entry = entry;
        }
    }
    if last_entry != *merged_entries.last().unwrap() /* FIX */ {
        merged_entries.push(last_entry);
    }
    let mut output = String::new();
    let mut line = 0;
    for (mut ip, mut count) in merged_entries.into_iter() {
        while count != 0 {
            let bits = math_log2(count);
            let b = min(min(count, 2u32.pow(bits)), 2u32.pow(ip.trailing_zeros()));
            write!(output, "{}/{}\n", Ipv4Addr::from(ip), 32 - bits).unwrap();
            line += 1;
            assert!(count > 0);
            count -= b;
            ip += b;
        }
    }
    console_log!("Total: {}/{}", line, raw_len);
    return Ok(output);
}
