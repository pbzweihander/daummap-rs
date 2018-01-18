//! Errors for `daummap` crate using `error-chain`

#![allow(missing_docs)]

error_chain!{
    foreign_links {
        Request(::reqwest::Error);
        Json(::serde_json::Error);
    }
    errors {
        /// When category group code parsing failed
        CategoryGroupParsingFailed {
            description("Category group parsing failed")
            display("Category group parsing failed")
        }
    }
}
