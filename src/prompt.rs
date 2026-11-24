use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::{fmt, io};

pub fn prompt<T>(msg: &str) -> T
where
    T: Promptable,
    <T as Promptable>::Err: Error + Debug + Send + Sync + 'static,
{
    loop {
        print!("{}? ", msg);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match T::from_input(input.trim()) {
            Ok(v) => return v,
            Err(e) => {
                eprintln!("\n{}", e);
            }
        }
    }
}

pub fn prompt_yes_no(msg: &str, default: bool) -> bool {
    let res: Empty<bool> =
        prompt(format!("{} [{}]", msg, if default { "Y/n" } else { "y/N" }).as_str());

    res.get_or(default)
}

pub fn prompt_with_default<T>(msg: &str, default: T) -> T
where
    T: Promptable + Display,
    <T as Promptable>::Err: Error + Debug + Send + Sync + 'static,
{
    let res: Empty<T> = prompt(format!("{} [{}]", msg, default).as_str());

    res.get_or(default)
}

pub trait Promptable: Sized {
    type Err;

    fn from_input(value: &str) -> Result<Self, Self::Err>;
}

impl Promptable for bool {
    type Err = ParseBoolError;

    fn from_input(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "y" | "yes" | "t" | "true" | "1" => Ok(true),
            "n" | "no" | "f" | "false" | "0" => Ok(false),
            _ => Err(ParseBoolError),
        }
    }
}

macro_rules! make_promptable {
    ($t:ty) => {
        impl Promptable for $t {
            type Err = InvalidInputError;

            fn from_input(value: &str) -> Result<Self, Self::Err> {
                match value.parse::<$t>() {
                    Ok(x) => Ok(x),
                    Err(e) => Err(InvalidInputError(e.to_string())),
                }
            }
        }
    };
}

make_promptable!(i8);
make_promptable!(i16);
make_promptable!(i32);
make_promptable!(i64);
make_promptable!(i128);
make_promptable!(u8);
make_promptable!(u16);
make_promptable!(u32);
make_promptable!(u64);
make_promptable!(u128);

impl Promptable for String {
    type Err = Infallible;

    fn from_input(value: &str) -> Result<Self, Self::Err> {
        Ok(value.to_string())
    }
}

impl<T> Promptable for Vec<T>
where
    T: Promptable,
    T::Err: Debug,
{
    type Err = T::Err;

    fn from_input(value: &str) -> Result<Self, Self::Err> {
        value
            .split(' ')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(T::from_input(s))
                }
            })
            .collect::<Result<Vec<T>, T::Err>>()
    }
}

struct Empty<T>(Option<T>);

impl<T> Empty<T> {
    pub fn get_or(self, default: T) -> T {
        self.0.unwrap_or(default)
    }
}

impl<T> Promptable for Empty<T>
where
    T: Promptable,
    T::Err: Debug,
{
    type Err = T::Err;

    fn from_input(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Ok(Empty(None));
        }

        T::from_input(s.trim()).map(|v| Empty(Some(v)))
    }
}

#[derive(Debug)]
pub struct ParseBoolError;

impl Display for ParseBoolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "provided string was not \"truthy\" or \"falsy\"")
    }
}

impl Error for ParseBoolError {}

#[derive(Debug)]
pub struct InvalidInputError(String);

impl Display for InvalidInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidInputError {}
