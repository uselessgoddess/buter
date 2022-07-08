#![doc = include_str!("../README.md")]

mod buter;
mod cell;

pub(crate) use cell::SyncUnsafeCell;

pub use self::buter::{Buter, ButerIter};
