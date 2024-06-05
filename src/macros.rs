#[macro_export]
macro_rules! bench {
    ($expr:expr) => {
        eprintln!("benching {}", stringify!($expr));
        let before = std::time::Instant::now();
        $expr;
        let after = std::time::Instant::now();
        eprintln!("took {:?}", after.duration_since(before));
    };
}
#[macro_export]
macro_rules! here {
    () => {
        eprintln!("here! [{}:{}]", file!(), line!());
    };
}
