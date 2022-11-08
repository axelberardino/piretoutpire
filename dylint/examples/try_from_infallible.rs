use std::str::FromStr;

struct Unit(());

impl From<()> for Unit {
    fn from(value: ()) -> Self {
        Self(value)
    }
}

impl FromStr for Unit {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(()))
    }
}

fn main() {
    let _ignored: Unit = ().try_into().expect("");
    let _ignored: Unit = "".parse().expect("");
    let _ignored = Unit::try_from(()).expect("");
    let _ignored = Unit::from_str("").expect("");
    let _ignored: Unit = Some(()).map(TryInto::try_into).expect("").expect("");
    let _ignored: Unit = Some(()).map(TryFrom::try_from).expect("").expect("");
}
