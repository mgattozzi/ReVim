error_chain! {
    foreign_links {
        Io(::std::io::Error) /// Error Chain variant of IO based errors
        ;
    }
}
