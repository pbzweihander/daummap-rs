//! Errors for `daummap` crate using `error-chain`

#![allow(missing_docs)]

error_chain!{
    foreign_links {
        Request(::reqwest::Error);
        Json(::serde_json::Error);
    }
    errors {
        /// When category group code parsing failed
        ParseCategoryGroup(s: String) {
            description("Cannot parse category group from string")
            display("Error during parsing category group from : {}", s)
        }
    }
}
