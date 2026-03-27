use lens::{Lenses, RefLens, lens};

#[derive(Clone, Lenses)]
struct Item {
    value: u32,
}

#[derive(Clone, Lenses)]
struct Root {
    items: Vec<Item>,
}

fn main() {
    let root = Root {
        items: vec![Item { value: 1 }, Item { value: 2 }],
    };
    let value_lens = lens!(Root.items[1].value);
    assert_eq!(*value_lens.get_ref(&root), 2);
}
