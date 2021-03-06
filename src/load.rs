/*! # Reading and loading hyphenation dictionaries

To hyphenate words in a given language, it is first necessary to load
the relevant hyphenation dictionary into memory. This module offers
convenience methods for common retrieval patterns, courtesy of the
[`Load`] trait.

```
use kl_hyphenate::Load;
use kl_hyphenate::{Standard, Language};
```

The primary function of [`Load`] is to deserialize dictionaries from
buffers – usually, file buffers.

```norun
use std::io;
use std::fs::File;

let path_to_dict = "/path/to/english-dictionary.bincode";
let dict_file = File::open(path_to_dict) ?;
let mut reader = io::BufReader::new(dict_file);
let english_us = Standard::from_reader(Language::EnglishUS, &mut reader) ?;
```

Dictionaries can be loaded from the file system rather more succintly with
the [`from_path`] shorthand:

```norun
let path = "dictionaries/en-us.standard.bincode";
let en_us = Standard::from_path(Language::EnglishUS, path) ?;
```

[`Load`]: trait.Load.html
[`from_path`]: trait.Load.html#method.from_path
*/

use bincode as bin;
use std::error;
use std::fmt;
use std::io;
use std::fs::File;
use std::path::Path;
use std::result;

use kl_hyphenate_commons::Language;
use kl_hyphenate_commons::dictionary::{Standard, Extended};

/// Convenience methods for the retrieval of hyphenation dictionaries.
pub trait Load : Sized {
    /// Read and deserialize the dictionary at the given path, verifying that it
    /// effectively belongs to the requested language.
    fn from_path<P>(lang : Language, path : P) -> Result<Self>
    where P : AsRef<Path> {
        let file = File::open(path) ?;
        Self::from_reader(lang, &mut io::BufReader::new(file))
    }

    /// Deserialize a dictionary from the provided reader, verifying that it
    /// effectively belongs to the requested language.
    fn from_reader<R>(lang : Language, reader : &mut R) -> Result<Self>
    where R : io::Read;

    /// Deserialize a dictionary from the provided reader.
    fn any_from_reader<R>(reader : &mut R) -> Result<Self>
    where R : io::Read;
}

macro_rules! impl_load {
    ($dict:ty, $suffix:expr) => {
        impl Load for $dict {
            fn from_reader<R>(lang : Language, reader : &mut R) -> Result<Self>
            where R : io::Read {
                let dict : Self = bin::config().limit(5_000_000).deserialize_from(reader) ?;
                let (found, expected) = (dict.language, lang);
                if found != expected {
                    Err(Error::LanguageMismatch { expected, found })
                } else { Ok(dict) }
            }

            fn any_from_reader<R>(reader : &mut R) -> Result<Self>
            where R : io::Read {
                let dict : Self = bin::config().limit(5_000_000).deserialize_from(reader) ?;
                Ok(dict)
            }
        }
    }
}

impl_load! { Standard, "standard" }
impl_load! { Extended, "extended" }


pub type Result<T> = result::Result<T, Error>;

/// Failure modes of dictionary loading.
#[derive(Debug)]
pub enum Error {
    /// The dictionary could not be deserialized.
    Deserialization(bin::Error),
    /// The dictionary could not be read.
    IO(io::Error),
    /// The loaded dictionary is for the wrong language.
    LanguageMismatch { expected : Language, found : Language },
    /// The embedded dictionary could not be retrieved.
    Resource
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Deserialization(ref e) => Some(e),
            Error::IO(ref e) => Some(e),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Deserialization(ref e) => e.fmt(f),
            Error::IO(ref e) => e.fmt(f),
            Error::LanguageMismatch { expected, found } =>
                write!(f, "\
Language mismatch: attempted to load a dictionary for `{}`, but found
a dictionary for `{}` instead.", expected, found),
            Error::Resource => f.write_str("the embedded dictionary could not be retrieved")
        }
    }
}

impl From<io::Error> for Error {
    fn from(err : io::Error) -> Error { Error::IO(err) }
}

impl From<bin::Error> for Error {
    fn from(err : bin::Error) -> Error { Error::Deserialization(err) }
}
