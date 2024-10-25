#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod v0 {
    include!(concat!(env!("OUT_DIR"), "/bindings/v0.rs"));
}