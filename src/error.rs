use std::error::Error;
use std::fmt::{Display, Formatter, Result as FMTResult};
use std::io::Error as IOError;
use Bencoding;

#[derive(Debug)]
pub enum DecodeError {
    IO(IOError),
    UnknownSymbol(char),
    LeadingZeroInteger,
    NegativeZeroInteger,
    InvalidNumberInteger(char),
    KeyNotStringDictionary(Bencoding),
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        match self {
            DecodeError::IO(e) => e.description(),
            DecodeError::UnknownSymbol(_) => {
                "Failed to match symbol to get type of bencoded data to parse"
            }
            DecodeError::LeadingZeroInteger => "A leading zero was read while parsing an integer",
            DecodeError::NegativeZeroInteger => {
                "Read a negative zero was read while parsing an integer"
            }
            DecodeError::InvalidNumberInteger(_) => {
                "Read a character that cannot be interpreted as a number"
            }
            DecodeError::KeyNotStringDictionary(_) => {
                "Expected a String as a key in a dictionary, found some other type"
            }
        }
    }
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter) -> FMTResult {
        match self {
            DecodeError::IO(e) => e.fmt(f),
            DecodeError::UnknownSymbol(n) => write!(f, "{} could not be understood", n),
            DecodeError::LeadingZeroInteger => write!(f, "Leading 0 before number"),
            DecodeError::NegativeZeroInteger => write!(f, "Negative zero"),
            DecodeError::InvalidNumberInteger(c) => write!(f, "{} is not a valid number", c),
            DecodeError::KeyNotStringDictionary(k) => {
                write!(f, "{:?} is not a correct key type", k)
            }
        }
    }
}

impl From<IOError> for DecodeError {
    fn from(e: IOError) -> DecodeError {
        DecodeError::IO(e)
    }
}
