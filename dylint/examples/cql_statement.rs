// this should throw an error
use scylla::statement::prepared_statement::PreparedStatement;

pub struct MyStruct {
    _query: PreparedStatement,
}

fn main() {}
