// // this should throw an error

// // one letter struct name.
// struct S;

// // struct with one letter members.
// struct Age {
//     a: i32,
// }

// // one letter enum name.
// enum E {}

// //enum with one letter values.
// enum Boolean {
//     T,
//     F,
// }

// // function with one letter parameter.
// fn print(a: i32) {
//     println!("{}", a);
// }

// fn main() {
//     // one letter parameter in lambda.
//     let _ = [1, 2, 3].iter().map(|a| a + 1);

//     // multiple one letter variables in lambda.
//     let _add = |a: i32, b: i32| a + b;

//     // one letter variables in tuples in lambda.
//     let _tuple_add = [(1, 2), (2, 3), (3, 4)].iter().map(|(a, b)| a + b);

//     // on letter variable in tuples in tuples in lambda.
//     let _tuple_add = [(1, (2, 3)), (2, (3, 4)), (3, (4, 5))]
//         .iter()
//         .map(|(a, (b, c))| a + b + c);

//     // one letter variable declaration.
//     let p = 3.14;

//     // one letter variable in an if-let expr.
//     if let Some(v) = Some(42) {}

//     // one letter variable in matching case.
//     match Some(42) {
//         Some(v) => {}
//         None => {}
//     }

//     // one letter variable in a loop definition.
//     for a in [1, 2, 3, 4] {}

//     // one letter variables in tuples in a loop definition.
//     for (a, b) in [(1, 2), (3, 4)] {}
// }

fn main() {}
