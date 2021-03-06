/// An error wrapper, contains a message for the user
/// and a cause to be logged.
pub struct Error {
    pub message: String,
    pub cause: Option<String>,
}

const DEBUG: bool = true;

impl Error {
    pub fn from_str<T>(message: &str) -> Result<T, Error> {
        Err(Error {
            message: message.to_owned(),
            cause: None,
        })
    }
}

pub trait Log {
    type T;

    fn unwrap_log(self) -> Self::T;
}

impl<T> Log for Result<T, Error> {
    type T = T;

    /// Unwrap the result. In case of error log and exit.
    fn unwrap_log(self) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                println!("{}", err.message);
                if DEBUG {
                    if let Some(cause) = err.cause {
                        println!("log: {}", cause)
                    }
                }
                std::process::exit(1)
            }
        }
    }
}

/// Wrap a Result<T, E> into a Result<T, Error>, where Error is Parrot
/// custom error.
pub fn wrap<T, E: std::fmt::Display>(err: Result<T, E>, message: &str) -> Result<T, Error> {
    match err {
        Ok(value) => Ok(value),
        Err(err) => Err(Error {
            message: message.to_owned(),
            cause: Some(format!("{}", err)),
        }),
    }
}
