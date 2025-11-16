/// Alias for `fmt!()`
#[macro_export]
macro_rules! fmt {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

// /// Alias for `.to_string()`
#[macro_export]
macro_rules! str {
    ($s:expr) => {
        $s.to_string()
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
