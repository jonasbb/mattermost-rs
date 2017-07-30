error_chain! {
    // Automatic conversions between this error chain and other
    // error chains. In this case, it will e.g. generate an
    // `ErrorKind` variant called `Another` which in turn contains
    // the `other_error::ErrorKind`, with conversions from
    // `other_error::Error`.
    //
    // Optionally, some attributes can be added to a variant.
    //
    // This section can be empty.
    links {
    }

    // Automatic conversions between this error chain and other
    // error types not defined by the `error_chain!`. These will be
    // wrapped in a new error with, in the first case, the
    // `ErrorKind::Fmt` variant. The description and cause will
    // forward to the description and cause of the original error.
    //
    // Optionally, some attributes can be added to a variant.
    //
    // This section can be empty.
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
        SerdeJson(::serde_json::Error);
        SerdeYaml(::serde_yaml::Error);
        Url(::url::ParseError);
    }

    // Define additional `ErrorKind` variants. The syntax here is
    // the same as `quick_error!`, but the `from()` and `cause()`
    // syntax is not supported.
    errors {
        InvalidOrMissingParameter
        MissingAccessToken
        MissingPermissions
        // InvalidOrMissingParameter(t: String) {
        //     description("The request has an invalid or missing parameter.")
        //     display("Invalid or missing parameter during '{}'", t)
        // }
        // MissingAccessToken(t: String) {
        //     description("The request is missing an access token.")
        //     display("Missing accesst token during '{}'", t)
        // }
    }
}
