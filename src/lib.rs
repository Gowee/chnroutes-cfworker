#![feature(type_alias_enum_variants)]
#![feature(async_await)]

#[macro_use]
mod utils;
mod rir_stats;

use std::{cmp::min, collections::HashSet, convert::TryFrom, fmt::Write, net::Ipv4Addr};

use cfg_if::cfg_if;
use isocountry::CountryCode;
use url::{form_urlencoded::Parse as QueryParam, Url};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, future_to_promise};
use web_sys::{Request, Response, ResponseInit};

use rir_stats::Registry;
use utils::{log, math_log2, set_panic_hook};

const HOMEPAGE: &'static str = "https://github.com/Gowee/chnroutes-cfworker/";

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, wasm-worker!".to_string()
}

#[derive(Debug)]
enum Error {
    RIRStatsMalformed(usize),
}

impl From<Error> for JsValue {
    fn from(err: Error) -> JsValue {
        match err {
            Error::RIRStatsMalformed(lineno) => {
                JsValue::from(format!("RIR stats is malformed at line {}.", lineno))
            }
        }
    }
}

#[derive(Debug)]
struct Options {
    countries: HashSet<CountryCode>,
    // whether country set is for excluding
    excluding: bool,
    /// `None` indicates to fetch stats from all five registries.
    registry: Registry,
}

#[wasm_bindgen]
pub async fn handleRequest(request: Request) ->  

pub async fn handle_request(request: Request) -> Response {
    let url = Url::parse(&request.url()).unwrap();
    match url.path() {
        "/" => {
            return Response::redirect_with_status(HOMEPAGE, 301).unwrap();
        }
        "/generate" => return handle_generate(request, url.query_pairs()).await,
        _ => {
            return Response::new_with_opt_str_and_init(None, ResponseInit::new().status(404))
                .unwrap()
        }
    }
    unimplemented!();
}

async fn handle_generate<'a>(request: Request, params: QueryParam<'a>) -> Response {
    let mut excluding = false;
    let mut countries = HashSet::new();
    let mut registry = None;
    for (param, value) in params {
        match param.as_ref() {
            "countries" => {
                if countries.len() > 0 {
                    return Response::new_with_opt_str_and_init(
                        Some("Invalid arguments: countries are given more than 1 time."),
                        ResponseInit::new().status(400),
                    )
                    .unwrap();
                }
                let value = if value.starts_with("!") {
                    excluding = true;
                    &value[1..]
                } else {
                    &value
                };
                for country in value.split("|") {
                    countries.insert(match CountryCode::for_alpha2_caseless(country) {
                        Ok(cc) => cc,
                        Err(e) => {
                            return Response::new_with_opt_str_and_init(
                                Some(&format!("Invalid country code: {}.", e)),
                                ResponseInit::new().status(400),
                            )
                            .unwrap()
                        }
                    });
                }
            }
            "registry" => {
                if registry.is_some() {
                    return Response::new_with_opt_str_and_init(
                        Some("Invalid arguments: registry are given more than 1 time."),
                        ResponseInit::new().status(400),
                    )
                    .unwrap();
                }
                registry = Some(match Registry::try_from(value.as_ref()) {
                    Ok(r) => r,
                    Err(e) => {
                        return Response::new_with_opt_str_and_init(
                            Some(&format!("Invalid registry: {}.", value)),
                            ResponseInit::new().status(400),
                        )
                        .unwrap()
                    }
                });
            }
        }
    }
    let options = Options {
        countries: countries,
        excluding: excluding,
        registry: registry.unwrap_or(Registry::All),
    };
    return Response::new_with_opt_str(Some(&format!("{:?}", options))).unwrap();
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
                let ip: Ipv4Addr = fields[3]
                    .parse()
                    .map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
                let count: u32 = fields[4]
                    .parse()
                    .map_err(|_e| Error::RIRStatsMalformed(index + 1))?;
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

/*
async function fetchRIRStats(registry) {
  // https://www.nro.net/about/rirs/statistics/
  const response = await fetch(`https://ftp.apnic.net/apnic/stats/${registry}/delegated-${registry}-latest`);
  if (!response.ok) {
    return new Error(`Failed to request upstream with HTTP ${response.status} (${response.statusText}).`);
  }
  return await response.text();
}
*/
