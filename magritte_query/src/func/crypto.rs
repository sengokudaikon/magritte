//! Crypto functions for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Crypto function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum CryptoFunction {
    /// Returns the blake3 hash of a value
    Blake3(String),
    /// Returns the md5 hash of a value
    Md5(String),
    /// Returns the sha1 hash of a value
    Sha1(String),
    /// Returns the sha256 hash of a value
    Sha256(String),
    /// Returns the sha512 hash of a value
    Sha512(String),

    // Argon2 functions
    /// Compares an argon2 hash to a password
    Argon2Compare(String, String),
    /// Generates a new argon2 hashed password
    Argon2Generate(String),

    // Bcrypt functions
    /// Compares a bcrypt hash to a password
    BcryptCompare(String, String),
    /// Generates a new bcrypt hashed password
    BcryptGenerate(String),

    // PBKDF2 functions
    /// Compares a pbkdf2 hash to a password
    Pbkdf2Compare(String, String),
    /// Generates a new pbkdf2 hashed password
    Pbkdf2Generate(String),

    // Scrypt functions
    /// Compares a scrypt hash to a password
    ScryptCompare(String, String),
    /// Generates a new scrypt hashed password
    ScryptGenerate(String),
}

impl Display for CryptoFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Basic hash functions
            Self::Blake3(val) => write!(f, "crypto::blake3({})", val),
            Self::Md5(val) => write!(f, "crypto::md5({})", val),
            Self::Sha1(val) => write!(f, "crypto::sha1({})", val),
            Self::Sha256(val) => write!(f, "crypto::sha256({})", val),
            Self::Sha512(val) => write!(f, "crypto::sha512({})", val),

            // Argon2 functions
            Self::Argon2Compare(hash, pass) => {
                write!(f, "crypto::argon2::compare({}, {})", hash, pass)
            }
            Self::Argon2Generate(pass) => write!(f, "crypto::argon2::generate({})", pass),

            // Bcrypt functions
            Self::BcryptCompare(hash, pass) => {
                write!(f, "crypto::bcrypt::compare({}, {})", hash, pass)
            }
            Self::BcryptGenerate(pass) => write!(f, "crypto::bcrypt::generate({})", pass),

            // PBKDF2 functions
            Self::Pbkdf2Compare(hash, pass) => {
                write!(f, "crypto::pbkdf2::compare({}, {})", hash, pass)
            }
            Self::Pbkdf2Generate(pass) => write!(f, "crypto::pbkdf2::generate({})", pass),

            // Scrypt functions
            Self::ScryptCompare(hash, pass) => {
                write!(f, "crypto::scrypt::compare({}, {})", hash, pass)
            }
            Self::ScryptGenerate(pass) => write!(f, "crypto::scrypt::generate({})", pass),
        }
    }
}

impl Callable for CryptoFunction {
    fn namespace() -> &'static str {
        "crypto"
    }

    fn category(&self) -> &'static str {
        match self {
            // Basic hash functions
            Self::Blake3(..)
            | Self::Md5(..)
            | Self::Sha1(..)
            | Self::Sha256(..)
            | Self::Sha512(..) => "hash",

            // Password functions
            Self::Argon2Compare(..) | Self::Argon2Generate(..) => "argon2",
            Self::BcryptCompare(..) | Self::BcryptGenerate(..) => "bcrypt",
            Self::Pbkdf2Compare(..) | Self::Pbkdf2Generate(..) => "pbkdf2",
            Self::ScryptCompare(..) | Self::ScryptGenerate(..) => "scrypt",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            // Compare functions return boolean and can be used in WHERE
            Self::Argon2Compare(..)
                | Self::BcryptCompare(..)
                | Self::Pbkdf2Compare(..)
                | Self::ScryptCompare(..)
        )
    }
}
