//! Env-var-substituted JSON configs.

use std::{collections::HashMap, ffi::OsString};

pub use serde_json;
pub use serde_json::Error as JsonError;

/// An error that can occur while templating the configuration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON error.
    #[error("json: {0}")]
    Json(JsonError),

    /// Environment variable error.
    #[error("env: {0}")]
    Env(EnvError),
}

/// An environment variable error.
#[derive(Debug, thiserror::Error)]
#[error("processing env var {env_var:?} error: {reason}")]
pub struct EnvError {
    /// The environment variable key.
    pub env_var: String,

    /// The error reason.
    pub reason: EnvErrorReason,
}

/// A reason for the environment variable error.
#[derive(Debug, thiserror::Error)]
pub enum EnvErrorReason {
    /// The value was non-unicode.
    #[error("the value is not valid unicode: {0:?}")]
    NotUnicode(OsString),
}

/// The configuration.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Config(HashMap<String, String>);

impl Config {
    /// Read and templatify the given input string as JSON using environment values with
    /// the given prefix as configuration values.
    pub fn templatify_from_env(input: &str, env_prefix: &str) -> Result<String, Error> {
        let mut config = Self::from_json(input).map_err(Error::Json)?;
        config.substitute_from_env(env_prefix).map_err(Error::Env)?;
        config.to_json().map_err(Error::Json)
    }

    /// Read the config from JSON string.
    ///
    /// This is a simple wrapper for [`serde_json`] invocation.
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    /// Read the config from JSON bytes.
    ///
    /// This is a simple wrapper for [`serde_json`] invocation.
    pub fn from_json_bytes(input: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(input)
    }

    /// Serialize the config to JSON string.
    ///
    /// This is a simple wrapper for [`serde_json`] invocation.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize the config to JSON bytes.
    ///
    /// This is a simple wrapper for [`serde_json`] invocation.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Write the JSON bytes into a given writer
    ///
    /// This is a simple wrapper for [`serde_json`] invocation.
    pub fn write_json(&self, w: impl std::io::Write) -> Result<(), serde_json::Error> {
        serde_json::to_writer(w, self)
    }

    /// Take a configuration and substitute it's values with the value of the corresponding
    /// environment variables, if present.
    pub fn substitute_from_env(&mut self, env_prefix: &str) -> Result<(), EnvError> {
        use convert_case::{Case, Casing};

        for (k, v) in self.0.iter_mut() {
            let env_suffix = k.to_case(Case::UpperSnake);
            let env_var = format!("{env_prefix}{env_suffix}");
            match std::env::var(&env_var) {
                Ok(val) => {
                    *v = val;
                }
                Err(std::env::VarError::NotPresent) => {
                    // Leave the current value as-is.
                }
                Err(std::env::VarError::NotUnicode(src)) => {
                    return Err(EnvError {
                        env_var,
                        reason: EnvErrorReason::NotUnicode(src),
                    })
                }
            }
        }

        Ok(())
    }

    /// Return an iterator over the config keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|s| s.as_str())
    }

    /// Return an iterator over the config key/values.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl From<HashMap<String, String>> for Config {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}

impl From<Config> for HashMap<String, String> {
    fn from(value: Config) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CONFIG: &str = indoc::indoc! { r##"
        {
            "sampleKey": "sample_value"
        }
    "## };
    const EXPECTED_CONFIG: &str = indoc::indoc! { r##"
        {
            "sampleKey": "changed_value"
        }
    "## };

    fn assert_json_eq(left: &str, right: &str) {
        let reformat_json = |input| {
            let value: serde_json::Value = serde_json::from_str(input).unwrap();
            serde_json::to_string_pretty(&value).unwrap()
        };

        assert_eq!(reformat_json(left), reformat_json(right));
    }

    #[test]
    fn happy_path() {
        assert!(
            std::env::var_os("SPA_CFG_TESTS_SAMPLE_KEY").is_none(),
            "test precondition failed"
        );
        std::env::set_var("SPA_CFG_TESTS_SAMPLE_KEY", "changed_value");

        let output = Config::templatify_from_env(SAMPLE_CONFIG, "SPA_CFG_TESTS_").unwrap();

        assert_json_eq(EXPECTED_CONFIG, &output);
    }
}
