use std::fmt;

pub trait ExpectWithContext<T> {
    /// Use with format_args! macro
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use lazin_test_utils::expect_ext::ExpectWithContext;
    /// let val = 42u64;
    /// let x: Result<u32, &str> = Err("emergency failure");
    /// x.expect_with_context(format_args!("Failed with dynamic value '{}'", val)); // panics with `Failed with dynamic value '42': emergency failure`
    /// ```
    fn expect_with_context(self, args: fmt::Arguments) -> T;
}

impl<T, E: fmt::Debug> ExpectWithContext<T> for Result<T, E> {
    #[inline]
    #[track_caller]
    fn expect_with_context(self, args: fmt::Arguments) -> T {
        match self {
            Ok(v) => v,
            Err(e) => unwrap_failed(&fmt::format(args), &e),
        }
    }
}

impl<T> ExpectWithContext<T> for Option<T> {
    #[inline]
    #[track_caller]
    fn expect_with_context(self, args: fmt::Arguments) -> T {
        match self {
            Some(v) => v,
            None => panic!("{}", &fmt::format(args)),
        }
    }
}

// Implementation copied from std
#[cfg(not(panic = "immediate-abort"))]
#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{msg}: {error:?}");
}

// Implementation copied from std
#[cfg(panic = "immediate-abort")]
#[inline]
#[cold]
#[track_caller]
const fn unwrap_failed<T>(_msg: &str, _error: &T) -> ! {
    panic!()
}
