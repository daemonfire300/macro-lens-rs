# macro-lens

`macro-lens` exports the `lens` Rust library crate for ergonomic, immutable data access and updates.

## Example

```rust
use lens::{Lens, RefLens, Lenses, lens};

#[derive(Clone, Lenses)]
struct Person {
    name: String,
}

let lens = lens!(Person.name);
let person = Person {
    name: "Pop Zeus".to_string(),
};

assert_eq!(lens.get_ref(&person), "Pop Zeus");
let updated = lens.set(person, "Melisande".to_string());
assert_eq!(updated.name, "Melisande");
```

## License

MIT.
