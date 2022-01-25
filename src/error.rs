use crate::mnemonic::Language;
use crate::utils::FeeInfo;
use base64::DecodeError as Base64DecodeError;
use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse;
use fmt::Debug;
use num_bigint::ParseBigIntError;
use prost::DecodeError;
use prost::EncodeError;
use secp256k1::Error as CurveError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Result as FmtResult;
use std::fmt::Result as FormatResult;
use std::num::ParseIntError;
use std::{error::Error, str::Utf8Error};
use std::{fmt, time::Duration};
use tonic::transport::Error as TonicError;
use tonic::Status;

#[derive(Debug)]
pub enum CosmosGrpcError {
    NoToken,
    BadResponse(String),
    BadStruct(String),
    SigningError { error: PrivateKeyError },
    ConnectionError { error: TonicError },
    RequestError { error: Status },
    DecodeError { error: DecodeError },
    BadInput(String),
    ChainNotRunning,
    NodeNotSynced,
    InvalidPrefix,
    NoBlockProduced { time: Duration },
    TransactionFailed { tx: TxResponse, time: Duration },
    InsufficientFees { fee_info: FeeInfo },
    ParseError { error: ParseBigIntError },
    InvalidAccount { type_url: String },
}

impl Display for CosmosGrpcError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            CosmosGrpcError::NoToken => {
                write!(f, "Account has no tokens! No details!")
            }
            CosmosGrpcError::BadResponse(val) => write!(f, "CosmosGrpc bad response {}", val),
            CosmosGrpcError::BadStruct(val) => {
                write!(f, "CosmosGrpc unexpected json returned {}", val)
            }
            CosmosGrpcError::BadInput(val) => write!(f, "CosmosGrpc bad input {}", val),
            CosmosGrpcError::DecodeError { error: val } => {
                write!(f, "CosmosGrpc bad any unpacking {}", val)
            }
            CosmosGrpcError::ConnectionError { error } => {
                write!(f, "CosmosGrpc Connection error {} {:?}", error, error)
            }
            CosmosGrpcError::RequestError { error } => {
                write!(f, "CosmosGrpc Request error {} {:?}", error, error)
            }
            CosmosGrpcError::ChainNotRunning => {
                write!(f, "CosmosGrpc this node is waiting on a blockchain start")
            }
            CosmosGrpcError::NodeNotSynced => {
                write!(f, "CosmosGrpc this node is syncing")
            }
            CosmosGrpcError::SigningError { error } => {
                write!(f, "CosmosGrpc could not sign using private key {:?}", error)
            }
            CosmosGrpcError::NoBlockProduced { time } => {
                write!(f, "CosmosGrpc NoBlockProduced in {}ms", time.as_millis())
            }
            CosmosGrpcError::InvalidPrefix => {
                write!(f, "CosmosGrpc InvalidPrefix")
            }
            CosmosGrpcError::TransactionFailed { tx, time } => {
                write!(
                    f,
                    "CosmosGrpc Transaction {:?} did not enter chain in {}ms",
                    tx,
                    time.as_millis()
                )
            }
            CosmosGrpcError::InsufficientFees { fee_info } => {
                write!(f, "Insufficient fees or gas for transaction {:?}", fee_info)
            }
            CosmosGrpcError::ParseError { error } => {
                write!(f, "Failed to Parse BigInt {:?}", error)
            }

            CosmosGrpcError::InvalidAccount { type_url } => {
                write!(f, "CosmosGrpc could not decode account: {}", type_url)
            }
        }
    }
}

impl Error for CosmosGrpcError {}

impl From<TonicError> for CosmosGrpcError {
    fn from(error: TonicError) -> Self {
        CosmosGrpcError::ConnectionError { error }
    }
}

impl From<Status> for CosmosGrpcError {
    fn from(error: Status) -> Self {
        CosmosGrpcError::RequestError { error }
    }
}

impl From<ArrayStringError> for CosmosGrpcError {
    fn from(_error: ArrayStringError) -> Self {
        CosmosGrpcError::InvalidPrefix
    }
}

impl From<DecodeError> for CosmosGrpcError {
    fn from(error: DecodeError) -> Self {
        CosmosGrpcError::DecodeError { error }
    }
}

impl From<PrivateKeyError> for CosmosGrpcError {
    fn from(error: PrivateKeyError) -> Self {
        CosmosGrpcError::SigningError { error }
    }
}

#[derive(Debug)]
pub enum AddressError {
    Bech32WrongLength,
    Bech32InvalidBase32,
    Bech32InvalidEncoding,
    HexDecodeError(ByteDecodeError),
    HexDecodeErrorWrongLength,
    PrefixTooLong(ArrayStringError),
    BytesDecodeErrorWrongLength,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddressError::Bech32WrongLength => write!(f, "Bech32WrongLength"),
            AddressError::Bech32InvalidBase32 => write!(f, "Bech32InvalidBase32"),
            AddressError::Bech32InvalidEncoding => write!(f, "Bech32InvalidEncoding"),
            AddressError::HexDecodeError(val) => write!(f, "HexDecodeError {}", val),
            AddressError::HexDecodeErrorWrongLength => write!(f, "HexDecodeError Wrong Length"),
            AddressError::PrefixTooLong(val) => write!(f, "Prefix too long {}", val),
            AddressError::BytesDecodeErrorWrongLength => write!(f, "BytesDecodeError Wrong Length"),
        }
    }
}

impl std::error::Error for AddressError {}

impl From<ArrayStringError> for AddressError {
    fn from(error: ArrayStringError) -> Self {
        AddressError::PrefixTooLong(error)
    }
}

impl From<bech32::Error> for AddressError {
    fn from(error: bech32::Error) -> Self {
        match error {
            bech32::Error::InvalidLength => AddressError::Bech32WrongLength,
            bech32::Error::InvalidChar(_) => AddressError::Bech32InvalidBase32,
            bech32::Error::InvalidData(_) => AddressError::Bech32InvalidEncoding,
            bech32::Error::InvalidChecksum => AddressError::Bech32InvalidEncoding,
            bech32::Error::InvalidPadding => AddressError::Bech32InvalidEncoding,
            bech32::Error::MixedCase => AddressError::Bech32InvalidEncoding,
            bech32::Error::MissingSeparator => AddressError::Bech32InvalidEncoding,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ByteDecodeError {
    DecodeError(Utf8Error),
    ParseError(ParseIntError),
}

impl Display for ByteDecodeError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ByteDecodeError::DecodeError(val) => write!(f, "ByteDecodeError {}", val),
            ByteDecodeError::ParseError(val) => write!(f, "ByteParseError {}", val),
        }
    }
}

impl Error for ByteDecodeError {}

#[derive(Debug)]
pub enum PublicKeyError {
    Bech32WrongLength,
    Bech32InvalidBase32,
    Bech32InvalidEncoding,
    HexDecodeError(ByteDecodeError),
    Base64DecodeError(Base64DecodeError),
    HexDecodeErrorWrongLength,
    BytesDecodeErrorWrongLength,
    PrefixTooLong(ArrayStringError),
}

impl fmt::Display for PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PublicKeyError::Bech32WrongLength => write!(f, "Bech32WrongLength"),
            PublicKeyError::Bech32InvalidBase32 => write!(f, "Bech32InvalidBase32"),
            PublicKeyError::Bech32InvalidEncoding => write!(f, "Bech32InvalidEncoding"),
            PublicKeyError::HexDecodeError(val) => write!(f, "HexDecodeError {}", val),
            PublicKeyError::Base64DecodeError(val) => write!(f, "Base64DecodeError {}", val),
            PublicKeyError::BytesDecodeErrorWrongLength => {
                write!(f, "BytesDecodeError Wrong Length")
            }
            PublicKeyError::HexDecodeErrorWrongLength => write!(f, "HexDecodeError Wrong Length"),
            PublicKeyError::PrefixTooLong(val) => write!(f, "Prefix too long {}", val),
        }
    }
}

impl std::error::Error for PublicKeyError {}

impl From<ArrayStringError> for PublicKeyError {
    fn from(error: ArrayStringError) -> Self {
        PublicKeyError::PrefixTooLong(error)
    }
}

impl From<bech32::Error> for PublicKeyError {
    fn from(error: bech32::Error) -> Self {
        match error {
            bech32::Error::InvalidLength => PublicKeyError::Bech32WrongLength,
            bech32::Error::InvalidChar(_) => PublicKeyError::Bech32InvalidBase32,
            bech32::Error::InvalidData(_) => PublicKeyError::Bech32InvalidEncoding,
            bech32::Error::InvalidChecksum => PublicKeyError::Bech32InvalidEncoding,
            bech32::Error::InvalidPadding => PublicKeyError::Bech32InvalidEncoding,
            bech32::Error::MixedCase => PublicKeyError::Bech32InvalidEncoding,
            bech32::Error::MissingSeparator => PublicKeyError::Bech32InvalidEncoding,
        }
    }
}

#[derive(Debug)]
pub enum PrivateKeyError {
    HexDecodeError(ByteDecodeError),
    HexDecodeErrorWrongLength,
    CurveError(CurveError),
    EncodeError(EncodeError),
    PublicKeyError(PublicKeyError),
    AddressError(AddressError),
    HdWalletError(HdWalletError),
}

impl fmt::Display for PrivateKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> FormatResult {
        match self {
            PrivateKeyError::HexDecodeError(val) => write!(f, "PrivateKeyError {}", val),
            PrivateKeyError::HexDecodeErrorWrongLength => write!(f, "PrivateKeyError Wrong Length"),
            PrivateKeyError::CurveError(val) => write!(f, "Secp256k1 Error {}", val),
            PrivateKeyError::EncodeError(val) => write!(f, "Could not encode message {}", val),
            PrivateKeyError::PublicKeyError(val) => write!(f, "{}", val),
            PrivateKeyError::AddressError(val) => write!(f, "{}", val),
            PrivateKeyError::HdWalletError(val) => write!(f, "{}", val),
        }
    }
}

impl std::error::Error for PrivateKeyError {}

impl From<CurveError> for PrivateKeyError {
    fn from(error: CurveError) -> Self {
        PrivateKeyError::CurveError(error)
    }
}

impl From<HdWalletError> for PrivateKeyError {
    fn from(error: HdWalletError) -> Self {
        PrivateKeyError::HdWalletError(error)
    }
}

impl From<PublicKeyError> for PrivateKeyError {
    fn from(error: PublicKeyError) -> Self {
        PrivateKeyError::PublicKeyError(error)
    }
}

impl From<AddressError> for PrivateKeyError {
    fn from(error: AddressError) -> Self {
        PrivateKeyError::AddressError(error)
    }
}

impl From<EncodeError> for PrivateKeyError {
    fn from(error: EncodeError) -> Self {
        PrivateKeyError::EncodeError(error)
    }
}

impl From<ByteDecodeError> for PrivateKeyError {
    fn from(error: ByteDecodeError) -> Self {
        PrivateKeyError::HexDecodeError(error)
    }
}

#[derive(Debug)]
pub enum HdWalletError {
    Bip39Error(Bip39Error),
    InvalidPathSpec(String),
}

impl fmt::Display for HdWalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> FormatResult {
        match self {
            HdWalletError::Bip39Error(val) => write!(f, "{}", val),
            HdWalletError::InvalidPathSpec(val) => write!(f, "HDWalletError invalid path {}", val),
        }
    }
}

impl std::error::Error for HdWalletError {}

/// A BIP39 error.
#[derive(Clone, PartialEq, Eq)]
pub enum Bip39Error {
    /// Mnemonic has a word count that is not a multiple of 6.
    BadWordCount(usize),
    /// Mnemonic contains an unknown word.
    UnknownWord(String),
    /// Entropy was not a multiple of 32 bits or between 128-256n bits in length.
    BadEntropyBitCount(usize),
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// The word list can be interpreted as multiple languages.
    AmbiguousWordList(Vec<Language>),
}

impl fmt::Display for Bip39Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Bip39Error::BadWordCount(c) => write!(
                f,
                "mnemonic has a word count that is not a multiple of 6: {}",
                c,
            ),
            Bip39Error::UnknownWord(ref w) => {
                write!(f, "mnemonic contains an unknown word: {}", w,)
            }
            Bip39Error::BadEntropyBitCount(c) => write!(
                f,
                "entropy was not between 128-256 bits or not a multiple of 32 bits: {} bits",
                c,
            ),
            Bip39Error::InvalidChecksum => write!(f, "the mnemonic has an invalid checksum"),
            Bip39Error::AmbiguousWordList(ref langs) => {
                write!(f, "ambiguous word list: {:?}", langs)
            }
        }
    }
}
impl Debug for Bip39Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug)]
pub enum ArrayStringError {
    TooLong,
}

impl Display for ArrayStringError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ArrayStringError::TooLong => {
                write!(f, "This string is too long!")
            }
        }
    }
}

impl Error for ArrayStringError {}
