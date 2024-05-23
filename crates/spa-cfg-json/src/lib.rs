//! Opinionated way of providing deployment time configuration to the Singe Page Apps with
//! env-var-substituted JSON.

use std::borrow::Cow;

/// The HTML templating engine for the SPA configuration.
#[derive(Debug)]
pub struct Engine {
    /// The prefix for the ENV vars to use.
    pub env_prefix: Cow<'static, str>,
}

/// The error type.
pub type Error = json_env_cfg::Error;

impl Engine {
    /// Apply the SPA configuration to the given JSON data.
    pub fn apply(&self, body: &mut Vec<u8>) -> Result<(), Error> {
        let mut config =
            json_env_cfg::Config::from_json_bytes(body).map_err(json_env_cfg::Error::Json)?;

        config
            .substitute_from_env(&self.env_prefix)
            .map_err(json_env_cfg::Error::Env)?;

        body.clear();
        config.write_json(body).map_err(json_env_cfg::Error::Json)?;

        Ok(())
    }
}
