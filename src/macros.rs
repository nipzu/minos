pub macro print($($arg:tt)*) {{
    use ::core::fmt::Write;
    #[allow(unused_unsafe)]
    write!(unsafe { $crate::console::CONSOLE.lock() }, $($arg)*).unwrap();
}}

pub macro println($($arg:tt)*) {{
    use ::core::fmt::Write;
    #[allow(unused_unsafe)]
    writeln!(unsafe { $crate::console::CONSOLE.lock() }, $($arg)*).unwrap();
}}
