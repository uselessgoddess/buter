#![doc = include_str!("../README.md")]
#![feature(generators)]
#![feature(iter_collect_into)]
#![feature(slice_index_methods)]

extern crate core;

mod buter;
pub use self::buter::{Buter, ButerIter};