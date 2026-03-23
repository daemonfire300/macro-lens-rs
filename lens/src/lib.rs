//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//
//
// Copyright (c) 2025 Julius Foitzik on derivative work
// All rights reserved.
//

#![doc = r#"
Macro-powered lenses for immutable Rust data structures.

## Example

```rust
use lens::{Lens, RefLens, Lenses, lens};

#[derive(Clone, Lenses)]
struct Address {
    street: String,
}

#[derive(Clone, Lenses)]
struct Person {
    address: Address,
}

let person = Person {
    address: Address {
        street: "123 Needmore Rd".to_string(),
    },
};

let street_lens = lens!(Person.address.street);
assert_eq!(street_lens.get_ref(&person), "123 Needmore Rd");

let updated = street_lens.set(person, "666 Titus Ave".to_string());
assert_eq!(street_lens.get_ref(&updated), "666 Titus Ave");
```

Vector indexing is supported as well:

```rust
use lens::{Lens, RefLens, Lenses, lens};

#[derive(Clone, Lenses)]
struct Item {
    value: u32,
}

#[derive(Clone, Lenses)]
struct Container {
    items: Vec<Item>,
}

let container = Container {
    items: vec![Item { value: 1 }, Item { value: 2 }],
};

let item_lens = lens!(Container.items[1].value);
assert_eq!(*item_lens.get_ref(&container), 2);
let updated = item_lens.set(container, 7);
assert_eq!(updated.items[1].value, 7);
```
"#]

extern crate self as lens;

mod core;
mod path;

pub use lens_derive::*;

#[macro_use]
mod macros;

pub use lens_macros::lens;

pub use core::*;
pub use path::*;
