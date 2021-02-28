use std::path::PathBuf;

// Module for errors using error_chain
// Might be expanded in the future

error_chain! {
    errors {
        ConfigParseError(path: PathBuf) {
            description("configuration file could not be parsed"),
            display("configuration file could not be parsed: '{}'", path.display()),
        }
    }
}

