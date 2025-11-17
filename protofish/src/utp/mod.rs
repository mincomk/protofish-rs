//! UTP module provides an interface of UTP specifications.

pub mod error;

mod protocol;
pub use protocol::*;
pub use tests::utp::mock_utp_pairs;

pub(crate) mod tests;
