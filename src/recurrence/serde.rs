use chrono::{NaiveDateTime};
use serde::{self, Deserialize, Serializer, Deserializer};
use serde::de::Error;
use super::RecurrenceRule;
use super::recurrence_parser;

pub fn serialize<S>(rule: &RecurrenceRule, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let string = format!("{}", rule);
    serializer.serialize_str(&string)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<RecurrenceRule, D::Error>
    where
        D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    recurrence_parser::parse(&string)
        .map_err(serde::de::Error::custom)
}