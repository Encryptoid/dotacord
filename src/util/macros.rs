/// Alias for `fmt!()`
// #[macro_export]
// macro_rules! format {
//     ($($arg:tt)*) => {
//         format!($($arg)*)
//     };
// }

// /// Alias for `.to_string()`
#[macro_export]
macro_rules! str {
    ($s:expr) => {
        $s.to_string()
    };
}

#[macro_export]
macro_rules! seq_span {
    ($name:expr) => {
        tracing::info_span!($name, span_name = $name)
    };
    ($name:expr, $($rest:tt)*) => {
        tracing::info_span!($name, span_name = $name, $($rest)*)
    };
}

// pub trait MapErrFmt<T> {
//     fn fmt_err(self, msg: impl std::fmt::Display) -> Result<T, String>;
// }

// impl<T, E: std::fmt::Display> MapErrFmt<T> for Result<T, E> {
//     fn fmt_err(self, msg: impl std::fmt::Display) -> Result<T, String> {
//         self.map_err(|e| fmt!("{} - {}", msg, e))
//     }
// }
