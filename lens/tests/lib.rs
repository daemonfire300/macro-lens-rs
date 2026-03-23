use lens::{Lens, LensPath, Lenses, RefLens, ValueLens, lens, vec_lens};

#[derive(Clone, Debug, PartialEq, Lenses)]
struct Address {
    street: String,
    city: String,
    postcode: String,
}

#[derive(Clone, Debug, PartialEq, Lenses)]
pub(crate) struct Person {
    name: String,
    age: u8,
    address: Address,
}

#[derive(Clone, Debug, PartialEq, Lenses)]
struct Team {
    members: Vec<Person>,
}

#[test]
fn a_simple_nested_data_structure_should_be_lensable() {
    let p0 = Person {
        name: "Pop Zeus".to_string(),
        age: 58,
        address: Address {
            street: "123 Needmore Rd".to_string(),
            city: "Dayton".to_string(),
            postcode: "99999".to_string(),
        },
    };

    let name_lens = lens!(Person.name);
    assert_eq!(name_lens.get_ref(&p0), "Pop Zeus");
    assert_eq!(lens!(Person.address.street).get_ref(&p0), "123 Needmore Rd");
    assert_eq!(name_lens.path(), LensPath::new(0));

    let p1 = lens!(Person.address.street).set(p0, "666 Titus Ave".to_string());
    assert_eq!(lens!(Person.name).get_ref(&p1), "Pop Zeus");
    assert_eq!(lens!(Person.address.street).get_ref(&p1), "666 Titus Ave");
}

#[test]
fn value_lenses_work_for_scalars_and_strings() {
    let person = Person {
        name: "Pop Zeus".to_string(),
        age: 58,
        address: Address {
            street: "123 Needmore Rd".to_string(),
            city: "Dayton".to_string(),
            postcode: "99999".to_string(),
        },
    };

    assert_eq!(PersonNameLens.get(&person), "Pop Zeus".to_string());
    assert_eq!(PersonAgeLens.get(&person), 58);
}

#[test]
fn vec_indexing_is_supported_in_macro_paths() {
    let team = Team {
        members: vec![
            Person {
                name: "Pop Zeus".to_string(),
                age: 58,
                address: Address {
                    street: "123 Needmore Rd".to_string(),
                    city: "Dayton".to_string(),
                    postcode: "99999".to_string(),
                },
            },
            Person {
                name: "Melisande".to_string(),
                age: 42,
                address: Address {
                    street: "1 Rustacean Way".to_string(),
                    city: "Berlin".to_string(),
                    postcode: "10115".to_string(),
                },
            },
        ],
    };

    let street_lens = lens!(Team.members[1].address.street);
    assert_eq!(street_lens.path(), LensPath::from_vec(vec![0, 1, 2, 0]));
    assert_eq!(street_lens.get_ref(&team), "1 Rustacean Way");

    let updated = street_lens.set(team, "2 Immutable Ave".to_string());
    assert_eq!(updated.members[1].address.street, "2 Immutable Ave");
}

#[test]
fn vec_lens_supports_primitives() {
    let scores = vec![3_u32, 5, 8];
    let second = vec_lens::<u32>(1);
    assert_eq!(*second.get_ref(&scores), 5);
    assert_eq!(second.set(scores, 13), vec![3, 13, 8]);
}
