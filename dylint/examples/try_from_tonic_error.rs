// this should throw an error
struct Duration(String);

impl TryFrom<String> for Duration {
    type Error = tonic::Status;

    fn try_from(_value: String) -> Result<Self, Self::Error> {
        unreachable!()
    }
}

fn main() {}
