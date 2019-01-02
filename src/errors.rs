//! Errors for `daummap` crate using `error-chain`

#![allow(missing_docs)]

use failure::Fail;

#[derive(Debug, Fail)]
#[fail(display = "Error during parsing category group from : {}", _0)]
pub struct ParseCategoryGroup(pub String);
