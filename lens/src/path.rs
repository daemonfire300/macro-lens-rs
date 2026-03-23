//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//
//
// Copyright (c) 2025 Julius Foitzik on derivative work
// All rights reserved.
//

use std::fmt;

/// An element in a `LensPath`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct LensPathElement {
    id: u64,
}

impl LensPathElement {
    pub fn new(id: u64) -> LensPathElement {
        LensPathElement { id }
    }
}

/// Describes a lens relative to a source data structure.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct LensPath {
    /// The path elements.
    pub elements: Vec<LensPathElement>,
}

impl LensPath {
    /// Creates a new `LensPath` with no elements.
    pub fn empty() -> LensPath {
        LensPath { elements: vec![] }
    }

    /// Creates a new `LensPath` with a single element.
    pub fn new(id: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement::new(id)],
        }
    }

    /// Creates a new `LensPath` with a single index (for an indexed type such as `Vec`).
    pub fn from_index(index: usize) -> LensPath {
        LensPath::new(index as u64)
    }

    /// Creates a new `LensPath` with two elements.
    pub fn from_pair(id0: u64, id1: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement::new(id0), LensPathElement::new(id1)],
        }
    }

    /// Creates a new `LensPath` from a vector of element identifiers.
    pub fn from_vec(ids: Vec<u64>) -> LensPath {
        LensPath {
            elements: ids.into_iter().map(LensPathElement::new).collect(),
        }
    }

    /// Creates a new `LensPath` that is the concatenation of the two paths.
    pub fn concat(lhs: LensPath, rhs: LensPath) -> LensPath {
        let mut elements = lhs.elements;
        elements.extend(rhs.elements);
        LensPath { elements }
    }
}

impl fmt::Debug for LensPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.elements
                .iter()
                .map(|elem| elem.id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{LensPath, LensPathElement};

    #[test]
    fn lens_path_constructors_should_work() {
        assert_eq!(LensPath::empty().elements, Vec::<LensPathElement>::new());
        assert_eq!(LensPath::new(4).elements, vec![LensPathElement::new(4)]);
        assert_eq!(LensPath::from_index(2), LensPath::new(2));
        assert_eq!(
            LensPath::from_pair(1, 3).elements,
            vec![LensPathElement::new(1), LensPathElement::new(3)]
        );
        assert_eq!(
            LensPath::from_vec(vec![1, 2, 3]).elements,
            vec![
                LensPathElement::new(1),
                LensPathElement::new(2),
                LensPathElement::new(3),
            ]
        );
    }

    #[test]
    fn lens_path_concat_should_work() {
        let p0 = LensPath::from_vec(vec![1, 2, 3]);
        let p1 = LensPath::from_vec(vec![4, 5]);
        let p2 = LensPath::concat(p0, p1);
        assert_eq!(p2, LensPath::from_vec(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn lens_path_debug_should_work() {
        let path = LensPath::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(format!("{path:?}"), "[1, 2, 3, 4, 5]");
    }
}
