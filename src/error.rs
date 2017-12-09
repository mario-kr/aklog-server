
error_chain! {
    errors {
        ConfigParseError(filename: String) {
            description("configuration file could not be parsed"),
            display("configuration file could not be parsed: '{}'", filename),
        }
        RegexParseError(regex: String) {
            description("regex could not be parsed"),
            display("regex not parsable: '{}'", regex),
        }
    }
}

