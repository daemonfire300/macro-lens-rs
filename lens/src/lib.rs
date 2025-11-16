//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

// Re-export the pl-lens-derive crate
pub use lens_derive::*;

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
pub use lens_macros::lens;

// The following is necessary to make exported macros visible.
#[macro_use]
pub mod macros;

pub mod lens;
pub mod path;
