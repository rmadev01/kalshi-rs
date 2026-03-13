#![allow(missing_docs)]

use serde::{Deserialize, Deserializer, Serializer};

use crate::error::Error;

pub const DOLLAR_SCALE: i64 = 10_000;
pub const COUNT_SCALE: i64 = 100;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FixedPointInput {
    String(String),
    Integer(i64),
    Float(f64),
}

fn parse_decimal_to_scaled(value: &str, scale: i64) -> Result<i64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("empty fixed-point value".to_string());
    }

    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed),
    };

    let (whole_part, frac_part) = match digits.split_once('.') {
        Some(parts) => parts,
        None => (digits, ""),
    };

    if whole_part.is_empty() && frac_part.is_empty() {
        return Err(format!("invalid fixed-point value: {trimmed}"));
    }
    if !whole_part.is_empty() && !whole_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid fixed-point whole part: {trimmed}"));
    }
    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid fixed-point fractional part: {trimmed}"));
    }

    let whole = if whole_part.is_empty() {
        0
    } else {
        whole_part
            .parse::<i64>()
            .map_err(|_| format!("invalid fixed-point whole part: {trimmed}"))?
    };

    let scale_digits = scale.to_string().len() - 1;
    let frac_len = frac_part.len();
    if frac_len > scale_digits {
        return Err(format!(
            "too many fractional digits for scale {scale}: {trimmed}"
        ));
    }

    let mut frac_string = frac_part.to_string();
    while frac_string.len() < scale_digits {
        frac_string.push('0');
    }
    let frac = if frac_string.is_empty() {
        0
    } else {
        frac_string
            .parse::<i64>()
            .map_err(|_| format!("invalid fixed-point fractional part: {trimmed}"))?
    };

    let mut scaled = whole
        .checked_mul(scale)
        .ok_or_else(|| format!("fixed-point overflow: {trimmed}"))?
        .checked_add(frac)
        .ok_or_else(|| format!("fixed-point overflow: {trimmed}"))?;

    if negative {
        scaled = -scaled;
    }

    Ok(scaled)
}

fn scaled_to_string(value: i64, scale: i64) -> String {
    let precision = scale.to_string().len() - 1;
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    let whole = abs / scale;
    let frac = abs % scale;
    format!("{sign}{whole}.{frac:0precision$}")
}

pub fn serialize_optional_dollars<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&scaled_to_string(*value, DOLLAR_SCALE)),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_optional_count<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&scaled_to_string(*value, COUNT_SCALE)),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_dollars<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match FixedPointInput::deserialize(deserializer)? {
        FixedPointInput::String(value) => {
            parse_decimal_to_scaled(&value, DOLLAR_SCALE).map_err(serde::de::Error::custom)
        }
        FixedPointInput::Integer(value) => Ok(value * 100),
        FixedPointInput::Float(value) => {
            let string = format!("{value:.4}");
            parse_decimal_to_scaled(&string, DOLLAR_SCALE).map_err(serde::de::Error::custom)
        }
    }
}

pub fn deserialize_optional_dollars<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<FixedPointInput>::deserialize(deserializer).and_then(|value| match value {
        Some(FixedPointInput::String(value)) if value.trim().is_empty() => Ok(None),
        Some(FixedPointInput::String(value)) => parse_decimal_to_scaled(&value, DOLLAR_SCALE)
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(FixedPointInput::Integer(value)) => Ok(Some(value * 100)),
        Some(FixedPointInput::Float(value)) => {
            let string = format!("{value:.4}");
            parse_decimal_to_scaled(&string, DOLLAR_SCALE)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        None => Ok(None),
    })
}

pub fn deserialize_count<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match FixedPointInput::deserialize(deserializer)? {
        FixedPointInput::String(value) => {
            parse_decimal_to_scaled(&value, COUNT_SCALE).map_err(serde::de::Error::custom)
        }
        FixedPointInput::Integer(value) => Ok(value * COUNT_SCALE),
        FixedPointInput::Float(value) => {
            let string = format!("{value:.2}");
            parse_decimal_to_scaled(&string, COUNT_SCALE).map_err(serde::de::Error::custom)
        }
    }
}

pub fn deserialize_optional_count<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<FixedPointInput>::deserialize(deserializer).and_then(|value| match value {
        Some(FixedPointInput::String(value)) if value.trim().is_empty() => Ok(None),
        Some(FixedPointInput::String(value)) => parse_decimal_to_scaled(&value, COUNT_SCALE)
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(FixedPointInput::Integer(value)) => Ok(Some(value * COUNT_SCALE)),
        Some(FixedPointInput::Float(value)) => {
            let string = format!("{value:.2}");
            parse_decimal_to_scaled(&string, COUNT_SCALE)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        None => Ok(None),
    })
}

pub fn parse_dollars(value: &str) -> Result<i64, Error> {
    parse_decimal_to_scaled(value, DOLLAR_SCALE).map_err(Error::Config)
}

pub fn parse_count(value: &str) -> Result<i64, Error> {
    parse_decimal_to_scaled(value, COUNT_SCALE).map_err(Error::Config)
}

pub fn format_dollars(value: i64) -> String {
    scaled_to_string(value, DOLLAR_SCALE)
}

pub fn format_count(value: i64) -> String {
    scaled_to_string(value, COUNT_SCALE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dollars() {
        assert_eq!(parse_dollars("0.5000").unwrap(), 5_000);
        assert_eq!(parse_dollars("1").unwrap(), 10_000);
    }

    #[test]
    fn parses_counts() {
        assert_eq!(parse_count("10.00").unwrap(), 1_000);
        assert_eq!(parse_count("3").unwrap(), 300);
    }

    #[test]
    fn formats_scaled_values() {
        assert_eq!(format_dollars(5_600), "0.5600");
        assert_eq!(format_count(250), "2.50");
    }
}
