//! A simple utility for parsing the HTTP headers from YAML.

use std::str::FromStr;

/// HTTP headers wrapper.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Headers(#[serde(with = "http_serde::header_map")] pub http::HeaderMap);

impl FromStr for Headers {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: Self = serde_yaml::from_str(s)?;
        Ok(val)
    }
}

impl From<http::HeaderMap> for Headers {
    fn from(value: http::HeaderMap) -> Self {
        Self(value)
    }
}

impl From<Headers> for http::HeaderMap {
    fn from(value: Headers) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let cases: &[(&'static str, &[(&'static str, &'static str)])] = &[
            ("", &[]),
            ("key: value", &[("key", "value")]),
            (
                "key: value\nkey1: value1",
                &[("key", "value"), ("key1", "value1")],
            ),
        ];

        for (sample, expected) in cases {
            let expected: http::HeaderMap = expected
                .iter()
                .map(|(k, v)| {
                    (
                        http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                        http::HeaderValue::from_bytes(v.as_bytes()).unwrap(),
                    )
                })
                .collect();

            let actual = Headers::from_str(sample).unwrap();

            assert_eq!(expected, actual.into());
        }
    }
}
