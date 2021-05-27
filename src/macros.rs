pub macro print($($arg:tt)*) {{
    use ::core::fmt::Write;
    write!($crate::console::CONSOLE.lock(), $($arg)*).unwrap();
}}

pub macro println($($arg:tt)*) {{
    use ::core::fmt::Write;
    writeln!($crate::console::CONSOLE.lock(), $($arg)*).unwrap();
}}
