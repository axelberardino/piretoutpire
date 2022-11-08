// this should throw an error for every one of those
use std::collections::{HashMap, HashSet};

struct Foo(String);

impl From<HashSet<String>> for Foo {
    fn from(value: HashSet<String>) -> Self {
        unreachable!()
    }
}

impl From<Vec<String>> for Foo {
    fn from(value: Vec<String>) -> Self {
        unreachable!()
    }
}

impl TryFrom<Vec<()>> for Foo {
    type Error = ();

    fn try_from(value: Vec<()>) -> Result<Self, Self::Error> {
        unreachable!()
    }
}

impl From<hashbrown::HashMap<String, String>> for Foo {
    fn from(value: hashbrown::HashMap<String, String>) -> Self {
        unreachable!()
    }
}

impl From<hashbrown::HashSet<String>> for Foo {
    fn from(value: hashbrown::HashSet<String>) -> Self {
        unreachable!()
    }
}

mod tests {
    struct Foo(String);
    impl From<Vec<String>> for Foo {
        fn from(value: Vec<String>) -> Self {
            unreachable!()
        }
    }
}

fn main() {}
