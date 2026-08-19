use std::cell::RefCell;

thread_local! {
    static LAZIN_FAILURES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[inline]
pub fn assert_failed(msg: String) {
    LAZIN_FAILURES.with(|f| f.borrow_mut().push(msg));
}

pub struct AssertGuard;

impl Default for AssertGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl AssertGuard {
    pub fn new() -> Self {
        assert!(
            LAZIN_FAILURES.with(|f| f.borrow().is_empty()),
            "expected thread_local LAZIN_FAILURES to be empty"
        );

        Self {}
    }
}

impl Drop for AssertGuard {
    fn drop(&mut self) {
        let assert_list =
            LAZIN_FAILURES.with(|f| f.borrow_mut().drain(..).collect::<Vec<String>>().join("\n"));

        if assert_list.is_empty() {
            return;
        }

        if std::thread::panicking() {
            eprintln!("Failed to assert on:\n{}", assert_list);
        } else {
            panic!("Failed to assert on:\n{}", assert_list);
        }
    }
}
