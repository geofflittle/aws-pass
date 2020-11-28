#[macro_export]
macro_rules! fatal_println {
    () => ({
        eprintln!();
        process::exit(1);
    });
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        process::exit(1);
    })
}
