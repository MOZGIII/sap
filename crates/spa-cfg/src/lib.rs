//! Opinionated way of providing deployment time configuration to the Singe Page Apps.

pub mod config;

use std::borrow::Cow;

/// The enforcement mode to use when applying the configuration.
#[derive(Debug)]
pub enum TemplateTagPresence {
    /// Require the script tag containing the configuration template to be present.
    ///
    /// If the tag is not found an error is returned.
    Required,

    /// Skip applying the configuration if the script containing the template is not found.
    SkipIfNotFound,
}

/// The settings for the SPA configuration system.
#[derive(Debug)]
pub struct SpaCfg {
    /// The prefix for the ENV vars to use.
    pub env_prefix: Cow<'static, str>,

    /// The requirements on the template tag presence.
    ///
    /// This can be used to allow
    pub template_tag_presence: TemplateTagPresence,
}

/// The error type.
pub type Error = html_templating::TemplatingError<config::Error>;

/// The content processor for the HTML templating.
struct ContentProcessor<'a>(&'a SpaCfg);

impl html_templating::ContentProcessor for ContentProcessor<'_> {
    type Error = config::Error;

    fn process(&self, input: &str) -> Result<String, Self::Error> {
        config::Config::templatify_from_env(input, &self.0.env_prefix)
    }
}

impl SpaCfg {
    /// Apply the SPA configuration to the given HTML document contents.
    pub fn apply(&self, body: &mut Vec<u8>) -> Result<(), Error> {
        let html_templating_processor = html_templating::Processor {
            script_type: Cow::Borrowed("application/spa-cfg"),
            content_processor: ContentProcessor(self),
        };

        let output = match html_templating_processor.process(body) {
            Ok(output) => output,
            Err(html_templating::TemplatingError::TemplateNotFound)
                if matches!(
                    self.template_tag_presence,
                    TemplateTagPresence::SkipIfNotFound
                ) =>
            {
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        *body = output;

        Ok(())
    }
}
