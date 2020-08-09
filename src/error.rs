pub struct Error {
    pub message: String,
    pub cause: Option<Box<dyn std::error::Error>>,
}

/// Unwrap the result. In case of error log and exit.
pub fn unwrap_log<T>(err: Result<T, Error>) -> T {
    match err {
        Ok(value) => value,
        Err(err) => {
            println!("{}", err.message);
            std::process::exit(1)
        }
    }
}

