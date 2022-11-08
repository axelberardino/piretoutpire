use std::{
    pin::Pin,
    task::{Context, Poll},
};

struct Unit(());

impl Iterator for Unit {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        unreachable!()
    }
}

impl futures::stream::Stream for Unit {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unreachable!()
    }
}

impl From<()> for Unit {
    fn from(_: ()) -> Self {
        unreachable!()
    }
}

fn main() {}
