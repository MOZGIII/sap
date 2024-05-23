//! The HTML templating logic.

use std::borrow::Cow;

/// The HTML teplating processor.
///
/// Will process the HTML code, and replace content of first the `script` node with the specified
/// type it encounters via the specified content processor.
#[derive(Debug)]
pub struct Processor<ContentProcessor> {
    /// The script type to work with.
    pub script_type: Cow<'static, str>,

    /// The logic to apply for script tag content processing.
    pub content_processor: ContentProcessor,
}

/// The template application error.
#[derive(Debug, thiserror::Error)]
pub enum TemplatingError<ContentProcessorError> {
    /// The template script was not found.
    #[error("template script not found in the HTML")]
    TemplateNotFound,

    /// The template script does not have content.
    #[error("template script does not have content")]
    NoContent,

    /// The template script content is not a text node.
    #[error("template script content is not a text node")]
    UnexpectedContent,

    /// Template content processor failed.
    #[error("template script content is not a text node")]
    ContentProcessor(ContentProcessorError),
}

/// An abstract content processor.
pub trait ContentProcessor {
    /// An error that could be encountered while processing the content.
    type Error;

    /// Process the provided content and get replacement content.
    fn process(&self, input: &str) -> Result<String, Self::Error>;
}

// pub fn noop(input: &str) -> String {
//     input.into()
// }

impl<T, E> ContentProcessor for T
where
    T: Fn(&str) -> Result<String, E>,
{
    type Error = E;

    fn process(&self, input: &str) -> Result<String, Self::Error> {
        (self)(input)
    }
}

/// Parse the raw document bytes into a useful form.
fn parse(html: &[u8]) -> kuchikiki::NodeRef {
    let parser = kuchikiki::parse_html();

    use kuchikiki::traits::TendrilSink as _;
    parser.from_utf8().one(html)
}

impl<ContentProcessor: self::ContentProcessor> Processor<ContentProcessor> {
    /// Process the HTML template.
    pub fn process(
        &self,
        html: &[u8],
    ) -> Result<Vec<u8>, TemplatingError<<ContentProcessor as self::ContentProcessor>::Error>> {
        let mut doc = parse(html);
        self.apply(&mut doc)?;

        let mut output = Vec::with_capacity(html.len());
        doc.serialize(&mut output).unwrap(); // vec write never fails

        Ok(output)
    }

    /// Apply the template processing.
    fn apply(
        &self,
        doc: &mut kuchikiki::NodeRef,
    ) -> Result<(), TemplatingError<<ContentProcessor as self::ContentProcessor>::Error>> {
        let selector = format!(
            r#"script[type="{}"]"#,
            self.script_type.replace(r#"""#, r#"\""#)
        );

        let el_data = doc
            .select_first(&selector)
            .map_err(|_| TemplatingError::TemplateNotFound)?;

        let el_node = el_data.as_node();

        let child = el_node.first_child().ok_or(TemplatingError::NoContent)?;

        let child_content = child
            .into_text_ref()
            .ok_or(TemplatingError::UnexpectedContent)?;

        let mut child_content_value = child_content.borrow_mut();

        let new_content = self
            .content_processor
            .process(&child_content_value)
            .map_err(TemplatingError::ContentProcessor)?;

        *child_content_value = new_content;

        Ok(())
    }
}
