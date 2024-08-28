use std::error::Error;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::string::ParseError;

#[derive(Debug)]
pub enum CoordinateError {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    InvalidDirection(char),
    InvalidLength(char),
}

impl fmt::Display for CoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CoordinateError::ParseIntError(e) => write!(f, "Failed to parse integer: {}", e),
            CoordinateError::ParseFloatError(e) => write!(f, "Failed to parse float: {}", e),
            CoordinateError::InvalidDirection(c) => write!(f, "Invalid direction: {}", c),
            CoordinateError::InvalidLength(c) => write!(f, "Invalid length: {}", c),
        }
    }
}

impl Error for CoordinateError {}

impl From<ParseIntError> for CoordinateError {
    fn from(err: ParseIntError) -> CoordinateError {
        CoordinateError::ParseIntError(err)
    }
}

impl From<ParseFloatError> for CoordinateError {
    fn from(err: ParseFloatError) -> CoordinateError {
        CoordinateError::ParseFloatError(err)
    }
}

pub struct Coordinate {
    degrees: u16,
    minutes: f32,
    direction: char,
}

impl Default for Coordinate {
    fn default() -> Self {
        Coordinate::new(0, 0.0, 'X')
    }
}

impl Coordinate {
    pub(crate) fn new(degrees: u16, minutes: f32, direction: char) -> Self {
        Coordinate {
            degrees,
            minutes,
            direction,
        }
    }

    pub fn to_decimal_degrees(&self) -> f64 {
        let mut decimal_degrees = self.degrees as f64 + (self.minutes as f64 / 60.0);
        if self.direction == 'S' || self.direction == 'W' {
            decimal_degrees *= -1.0;
        }
        decimal_degrees
    }

    pub fn to_string(&self) -> String {
        format!("{}° {}' {}", self.degrees, self.minutes, self.direction)
    }

    pub fn from_latitude_string(coord: String, direction: char) -> Result<Self, CoordinateError> {
        if coord.len() < 4 {
            return Err(CoordinateError::InvalidLength(direction));
        }
        let degrees = coord[..2].parse::<u16>()?;
        let minutes = coord[2..].parse::<f32>()?;
        if direction != 'N' && direction != 'S' {
            return Err(CoordinateError::InvalidDirection(direction));
        }
        Ok(Self {
            degrees,
            minutes,
            direction,
        })
    }

    pub fn from_longitude_string(coord: String, direction: char) -> Result<Self, CoordinateError> {
        if coord.len() < 5 {
            return Err(CoordinateError::InvalidLength(direction));
        }
        let degrees = coord[..3].parse::<u16>()?;
        let minutes = coord[3..].parse::<f32>()?;
        if direction != 'E' && direction != 'W' {
            return Err(CoordinateError::InvalidDirection(direction));
        }
        Ok(Self {
            degrees,
            minutes,
            direction,
        })
    }
}
