#[macro_export]
macro_rules! assert_err {
    ($err:expr, $pattern:pat $(if $cond:expr)?) => {
        if !matches!($err, $pattern $(if $cond)?) {
            panic!(
                "Expected {}, got {:?}",
                stringify!($pattern $(if $cond)?),
                $err
            );
        }
    };
}

mod test_config;
mod test_context;

pub use test_context::TestContext;
