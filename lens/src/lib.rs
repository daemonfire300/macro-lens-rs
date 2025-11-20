//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//
//
// Copyright (c) 2025 Julius Foitzik on derivate work
// All rights reserved.
//

extern crate self as lens;

mod core;
mod path;
/// This is a macro-based shorthand that allows us to write:
///
/// ```text,no_run
///   lens!(SomeStruct.foo.bar_vec[3].baz)
/// ```
///
/// instead of:
///
/// ```text,no_run
///   compose_lens!(SomeStructFooLens, FooBarVecLens, vec_lens::<BarThing>(3), BarThingBazLens)
/// ```
// TODO(juf): Re-work so original functionality is restored
pub use lens_derive::*;

#[macro_use]
mod macros;

pub use lens_macros::lens;

pub use core::*;
pub use path::*;
