use lens::{Lens, RefLens, Lenses, lens};

#[derive(Clone, Lenses)]
pub(crate) struct Person {
    address: Address,
}

#[derive(Clone, Lenses)]
pub(crate) struct Address {
    street: String,
}

fn main() {
    let person = Person {
        address: Address {
            street: "Needmore".to_string(),
        },
    };
    let street_lens = lens!(Person.address.street);
    let updated = street_lens.set(person, "Titus".to_string());
    assert_eq!(street_lens.get_ref(&updated), "Titus");
}
