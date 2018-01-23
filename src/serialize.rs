use std::collections::HashSet;
use std::hash::Hash;

pub fn deserialize_embedded_json<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: ::serde::de::Deserializer<'de>,
    T: ::serde::de::DeserializeOwned,
{
    use serde::de::*;
    use serde_json;
    use std::fmt;
    use std::marker::PhantomData;
    #[derive(Default)]
    struct Helper<S: DeserializeOwned>(PhantomData<S>);

    impl<'de, S> Visitor<'de> for Helper<S>
    where
        S: DeserializeOwned,
    {
        type Value = S;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "valid json object")
        }

        fn visit_str<E>(self, value: &str) -> Result<S, E>
        where
            E: Error,
        {
            serde_json::from_str(value).map_err(Error::custom)
        }
    }

    deserializer.deserialize_any(Helper(PhantomData))
}

use std::fmt::Debug;
pub fn deserialize_string_set<'de, D, T>(deserializer: D) -> Result<HashSet<T>, D::Error>
where
    D: ::serde::de::Deserializer<'de>,
    T: ::serde::de::DeserializeOwned,
    T: Debug,
    T: Hash,
    T: Eq,
{
    use serde::de::*;
    use serde_json;
    use std::fmt;
    use std::marker::PhantomData;
    #[derive(Default)]
    struct Helper<S: DeserializeOwned>(PhantomData<S>);

    impl<'de, S> Visitor<'de> for Helper<S>
    where
        S: DeserializeOwned,
        S: Debug,
        S: Hash,
        S: Eq,
    {
        type Value = HashSet<S>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "valid json object")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            value
                .split_whitespace()
                .map(|v| serde_json::from_str(&*format!("\"{}\"", v)))
                .collect::<Result<_, _>>()
                .map_err(Error::custom)
        }
    }

    deserializer.deserialize_str(Helper(PhantomData))
}

pub mod ts_seconds {
    use serde::{de, ser};
    use std::fmt;

    use chrono::{DateTime, FixedOffset, Utc};
    use chrono::offset::{LocalResult, TimeZone};

    /// Deserialize a `DateTime` from a milliseconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # // We mark this ignored so that we can test on 1.13 (which does not
    /// # // support custom derive), and run tests with --ignored on beta and
    /// # // nightly to actually trigger these.
    /// #
    /// # #[macro_use] extern crate serde_derive;
    /// # #[macro_use] extern crate serde_json;
    /// # extern crate chrono;
    /// # use chrono::{DateTime, Utc};
    /// use chrono::serde::ts_seconds::deserialize as from_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_ts")]
    ///     time: DateTime<Utc>
    /// }
    ///
    /// # fn example() -> Result<S, serde_json::Error> {
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1431684000 }"#)?;
    /// # Ok(my_s)
    /// # }
    /// # fn main() { example().unwrap(); }
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(try!(
            d.deserialize_i64(MillisecondsTimestampVisitor)
                .map(|dt| dt.with_timezone(&Utc))
        ))
    }

    /// Serialize a UTC datetime into an integer number of milliseconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # // We mark this ignored so that we can test on 1.13 (which does not
    /// # // support custom derive), and run tests with --ignored on beta and
    /// # // nightly to actually trigger these.
    /// #
    /// # #[macro_use] extern crate serde_derive;
    /// # #[macro_use] extern crate serde_json;
    /// # extern crate chrono;
    /// # use chrono::{TimeZone, DateTime, Utc};
    /// use chrono::serde::ts_seconds::serialize as to_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_ts")]
    ///     time: DateTime<Utc>
    /// }
    ///
    /// # fn example() -> Result<String, serde_json::Error> {
    /// let my_s = S {
    ///     time: Utc.ymd(2015, 5, 15).and_hms(10, 0, 0),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1431684000}"#);
    /// # Ok(as_string)
    /// # }
    /// # fn main() { example().unwrap(); }
    /// ```
    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.timestamp() * 1000 + dt.timestamp_subsec_millis() as i64)
    }

    struct MillisecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for MillisecondsTimestampVisitor {
        type Value = DateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a unix timestamp in milliseconds")
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_i64<E>(self, value: i64) -> Result<DateTime<FixedOffset>, E>
        where
            E: de::Error,
        {
            from(
                FixedOffset::east(0).timestamp_opt(value / 1000, (value % 1000 * 1_000_000) as u32),
                &value,
            )
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_u64<E>(self, value: u64) -> Result<DateTime<FixedOffset>, E>
        where
            E: de::Error,
        {
            from(
                FixedOffset::east(0)
                    .timestamp_opt((value / 1000) as i64, (value % 1000 * 1_000_000) as u32),
                &value,
            )
        }
    }

    // try!-like function to convert a LocalResult into a serde-ish Result
    fn from<T, E, V>(me: LocalResult<T>, ts: &V) -> Result<T, E>
    where
        E: de::Error,
        V: fmt::Display,
        T: fmt::Display,
    {
        match me {
            LocalResult::None => Err(E::custom(format!("value is not a legal timestamp: {}", ts))),
            LocalResult::Ambiguous(min, max) => Err(E::custom(format!(
                "value is an ambiguous timestamp: {}, could be either of {}, {}",
                ts, min, max
            ))),
            LocalResult::Single(val) => Ok(val),
        }
    }
}

pub mod option_ts_milliseconds {
    //FIXME comments
    use serde::{de, ser};
    use std::fmt;

    use chrono::{DateTime, FixedOffset, Utc};
    use chrono::offset::{LocalResult, TimeZone};

    /// Deserialize a `DateTime` from a milliseconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # // We mark this ignored so that we can test on 1.13 (which does not
    /// # // support custom derive), and run tests with --ignored on beta and
    /// # // nightly to actually trigger these.
    /// #
    /// # #[macro_use] extern crate serde_derive;
    /// # #[macro_use] extern crate serde_json;
    /// # extern crate chrono;
    /// # use chrono::{DateTime, Utc};
    /// use chrono::serde::ts_seconds::deserialize as from_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_ts")]
    ///     time: DateTime<Utc>
    /// }
    ///
    /// # fn example() -> Result<S, serde_json::Error> {
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1431684000 }"#)?;
    /// # Ok(my_s)
    /// # }
    /// # fn main() { example().unwrap(); }
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Some(d.deserialize_i64(MillisecondsTimestampVisitor)
            .map(|dt| dt.with_timezone(&Utc))?))
    }

    /// Serialize a UTC datetime into an integer number of milliseconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # // We mark this ignored so that we can test on 1.13 (which does not
    /// # // support custom derive), and run tests with --ignored on beta and
    /// # // nightly to actually trigger these.
    /// #
    /// # #[macro_use] extern crate serde_derive;
    /// # #[macro_use] extern crate serde_json;
    /// # extern crate chrono;
    /// # use chrono::{TimeZone, DateTime, Utc};
    /// use chrono::serde::ts_seconds::serialize as to_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_ts")]
    ///     time: DateTime<Utc>
    /// }
    ///
    /// # fn example() -> Result<String, serde_json::Error> {
    /// let my_s = S {
    ///     time: Utc.ymd(2015, 5, 15).and_hms(10, 0, 0),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1431684000}"#);
    /// # Ok(as_string)
    /// # }
    /// # fn main() { example().unwrap(); }
    /// ```
    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if let &Some(dt) = dt {
            serializer.serialize_i64(dt.timestamp() * 1000 + dt.timestamp_subsec_millis() as i64)
        } else {
            serializer.serialize_unit()
        }
    }

    struct MillisecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for MillisecondsTimestampVisitor {
        type Value = DateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a unix timestamp in milliseconds")
        }

        // FIXME comments
        // also in copy above

        // FIXME convert to self::value above

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            from(
                FixedOffset::east(0).timestamp_opt(value / 1000, (value % 1000 * 1_000_000) as u32),
                &value,
            )
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            from(
                FixedOffset::east(0)
                    .timestamp_opt((value / 1000) as i64, (value % 1000 * 1_000_000) as u32),
                &value,
            )
        }
    }

    // try!-like function to convert a LocalResult into a serde-ish Result
    fn from<T, E, V>(me: LocalResult<T>, ts: &V) -> Result<T, E>
    where
        E: de::Error,
        V: fmt::Display,
        T: fmt::Display,
    {
        match me {
            LocalResult::None => Err(E::custom(format!("value is not a legal timestamp: {}", ts))),
            LocalResult::Ambiguous(min, max) => Err(E::custom(format!(
                "value is an ambiguous timestamp: {}, could be either of {}, {}",
                ts, min, max
            ))),
            LocalResult::Single(val) => Ok(val),
        }
    }
}
