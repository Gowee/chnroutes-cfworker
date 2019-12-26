extern crate cfg_if;
extern crate wasm_bindgen;

#[macro_use]
mod utils;

use cfg_if::cfg_if;
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
use wasm_bindgen::prelude::*;

use utils::{log, set_panic_hook, MathLog2};

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
    RIRStatsMalformed(usize),
    RoutesEmpty
}

impl From<Error> for JsValue {
    fn from(err: Error) -> JsValue {
        match err {
            Error::RIRStatsMalformed(lineno) => {
                JsValue::from(format!("RIR stats is malformed at line {}.", lineno))
            }
            Error::RoutesEmpty => {
                JsValue::from(format!("Either the upstream stats is empty or all records are filtered out."))
            }
        }
    }
}

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, wasm-worker!".to_string()
}

fn parse_contries(countries: &str) -> (bool, HashSet<&str>) {
    let excluding;
    let country_set: HashSet<&str> = if countries.starts_with("!") {
        excluding = true;
        &countries[1..]
    } else {
        excluding = false;
        countries
    }
    .split(",")
    .collect();

    return (excluding, country_set);
}

macro_rules! decimal_type {
    (Ipv4Addr) => {
        u32
    };
    (Ipv6Addr) => {
        u128
    };
}

macro_rules! type_name {
    (Ipv4Addr) => {
        "ipv4"
    };
    (Ipv6Addr) => {
        "ipv6"
    };
}

macro_rules! parse_count {
    (Ipv4Addr, $raw:expr) => {
        $raw
    };
    (Ipv6Addr, $raw:expr) => {
        1 << (128 - $raw)
    };
}

macro_rules! implement_routes_from_rir_stats {
    ($addr_type:ident, $name:ident) => {
        #[wasm_bindgen]
        pub fn $name(raw_data: &str, countries: &str) -> Result<String, JsValue> {
            set_panic_hook();
            let (excluding, country_set) = parse_contries(countries);
            // https://www.apnic.net/about-apnic/corporate-documents/documents/resource-guidelines/rir-statistics-exchange-format/
            let mut raw_entries: Vec<(decimal_type!($addr_type), decimal_type!($addr_type))> =
                Vec::new();
            for (index, entry) in raw_data.split("\n").enumerate() {
                if !entry.starts_with("#") && !entry.is_empty() {
                    let fields: Vec<&str> = entry.split("|").collect();
                    if fields.len() >= 7
                        && (excluding ^ country_set.contains(fields[1]))
                        && fields[2] == type_name!($addr_type)
                    {
                        let ip: $addr_type = fields[3]
                            .parse()
                            .map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
                        let value: decimal_type!($addr_type) = fields[4]
                            .parse()
                            .map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
                        let count: decimal_type!($addr_type) = parse_count!($addr_type, value);
                        let ip = ip.into();
                        raw_entries.push((ip, count));
                    }
                }
            }
            let raw_len = raw_entries.len();
            raw_entries.sort();
            let mut merged_entries = Vec::new();
            let mut last_entry = *raw_entries.first().ok_or(Error::RoutesEmpty)?;
            for entry in raw_entries.into_iter().skip(1) {
                if entry.0 - last_entry.0 == last_entry.1 {
                    last_entry = (last_entry.0, last_entry.1 + entry.1)
                } else {
                    merged_entries.push(last_entry);
                    last_entry = entry;
                }
            }
            if last_entry != *merged_entries.last().unwrap()
            /* FIX */
            {
                merged_entries.push(last_entry);
            }
            let mut output = String::new();
            let mut line = 0;
            for (mut ip, mut count) in merged_entries.into_iter() {
                console_log!("{} {}", $addr_type::from(ip), count);
                while count != 0 {
                    let b = min(
                        (2 as decimal_type!($addr_type)).pow(count.log2()),
                        (2 as decimal_type!($addr_type)).pow(ip.trailing_zeros()),
                    );
                    write!(
                        output,
                        "{}/{}\n",
                        $addr_type::from(ip),
                        std::mem::size_of::<decimal_type!($addr_type)>() as u32 * 8 - b.log2()
                    )
                    .unwrap();
                    line += 1;
                    assert!(count > 0);
                    count -= b;
                    ip += b;
                }
            }
            console_log!("Total: {}/{}", line, raw_len);
            return Ok(output);
        }
    };
}

implement_routes_from_rir_stats!(Ipv4Addr, routes_from_rir_stats);
implement_routes_from_rir_stats!(Ipv6Addr, routes_from_rir_stats6);
