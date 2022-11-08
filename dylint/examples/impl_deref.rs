// this should throw an error
struct Duration(String);

impl std::ops::Deref for Duration {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

fn main() {}
