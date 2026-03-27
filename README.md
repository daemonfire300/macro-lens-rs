# macro-lens

`macro-lens` provides macro-powered lenses for immutable Rust data structures.

The workspace currently contains three crates:

- `macro-lens`: the main library crate, exported to Rust code as `lens`
- `macro-lens-derive`: the `#[derive(Lenses)]` proc-macro crate
- `macro-lens-macros`: the `lens!(...)` proc-macro crate

## Usage

Add the main crate to your `Cargo.toml`:

```toml
[dependencies]
macro-lens = "2"
```

Then import the re-exported macros and traits from the `lens` library crate:

```rust
use lens::{Lens, RefLens, Lenses, lens};
```

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
assert_eq!(updated.address.street, "666 Titus Ave");
```

Vector indexing is supported too:

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

## Development

The repo is flake-based. The canonical validation command is:

```sh
nix flake check --print-build-logs
```

For a short local alias to the same validation contract, run:

```sh
nix run .#ci
```

For interactive development, a typical shell entrypoint is:

```sh
nix develop -c zsh
```

The flake exposes build and validation checks for the Rust workspace, and GitHub Actions uses the same Nix entrypoints.

## Contribution

### Note on Publishing

The release order matters because the main crate depends on the two proc-macro crates:

1. Publish `macro-lens-derive`
2. Publish `macro-lens-macros`
3. Publish `macro-lens`

The workspace checks validate `cargo package` for all three crates and `cargo publish --dry-run` for the proc-macro crates. The final `macro-lens` upload can only be fully dry-run against crates.io after the dependency crates are available there at the matching version.

## License

MIT. See [LICENSE](LICENSE).
