// this should throw an error
struct Duration(String);

impl From<String> for Duration {
    fn from(inner: String) -> Self {
        Self(inner)
    }
}

fn main() {}
