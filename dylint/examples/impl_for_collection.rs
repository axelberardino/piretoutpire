// this should throw an error for every one of those
use std::collections::{HashMap, HashSet};

struct Foo(String);

impl From<Foo> for HashSet<Foo> {
    fn from(value: Foo) -> Self {
        unreachable!()
    }
}

impl From<Foo> for Vec<Foo> {
    fn from(value: Foo) -> Self {
        unreachable!()
    }
}

impl From<Foo> for hashbrown::HashMap<Foo, Foo> {
    fn from(value: Foo) -> Self {
        unreachable!()
    }
}

impl From<Foo> for hashbrown::HashSet<Foo> {
    fn from(value: Foo) -> Self {
        unreachable!()
    }
}

impl TryFrom<Foo> for Vec<String> {
    type Error = ();

    fn try_from(value: Foo) -> Result<Self, Self::Error> {
        unreachable!()
    }
}

mod tests {
    struct Foo(String);

    impl From<Foo> for hashbrown::HashSet<Foo> {
        fn from(value: Foo) -> Self {
            unreachable!()
        }
    }
}

fn main() {}
