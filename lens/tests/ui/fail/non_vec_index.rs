use lens::{Lenses, lens};

#[derive(Lenses)]
struct Root {
    value: u32,
}

fn main() {
    let _ = lens!(Root.value[0]);
}
