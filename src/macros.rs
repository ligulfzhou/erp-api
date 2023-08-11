#[macro_export]
macro_rules! ty {
    ($type:ty) => {{
        let result = std::any::type_name::<$type>();
        match result.rsplit_once(':') {
            Some((_, s)) => s,
            None => result,
        }
    }};
}
