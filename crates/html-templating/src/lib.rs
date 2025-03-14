//! The HTML templating logic.

mod dom;

use html5ever::tendril::TendrilSink as _;
use markup5ever_rcdom as rcdom;
use std::borrow::Cow;

pub use dom::{template_element_filter, TemplateElementFilter};

/// The HTML templating processor.
///
/// Will process the HTML code, and replace content of first the `script` node with the specified
/// type it encounters via the specified content processor.
#[derive(Debug)]
pub struct Processor<TemplateElementFilter, ContentProcessor> {
    /// The element filter to select the template to work with.
    pub template_element_filter: TemplateElementFilter,

    /// The logic to apply for script tag content processing.
    pub content_processor: ContentProcessor,
}

/// The template application error.
#[derive(Debug, thiserror::Error)]
pub enum TemplatingError<ContentProcessorError> {
    /// The HTML parsing has failed.
    #[error("HTML parsing failed")]
    HtmlParsing(Vec<Cow<'static, str>>),

    /// The template script was not found.
    #[error("template script not found in the HTML")]
    TemplateNotFound,

    /// The template script content was not found.
    #[error("template script content not found")]
    TemplateContentNotFound,

    /// The template element has more than one child.
    #[error("template element has more than one child")]
    TemplateElementHasMoreThanOneChild,

    /// Error during template application.
    #[error("template application: {0}")]
    TemplateApplication(TemplateApplicationError<ContentProcessorError>),
}

/// The template application error.
#[derive(Debug, thiserror::Error)]
pub enum TemplateApplicationError<ContentProcessorError> {
    /// No text content found.
    #[error("no text content")]
    TemplateNonTextContent,

    /// Error from the content processor.
    #[error("content processor error: {0}")]
    ContentProcessor(ContentProcessorError),
}

/// Applies the template to the provided DOM handle using the content processor.
///
/// # Parameters
/// - `handle`: The DOM handle to apply the template to.
/// - `content_processor`: The content processor to use for processing the template content.
///
/// # Returns
/// - `Ok(())` if the template was successfully applied.
/// - `Err(TemplateApplicationError)` if an error occurred during template application.
fn apply_template<ContentProcessor: self::ContentProcessor>(
    handle: &dom::Handle,
    content_processor: &ContentProcessor,
) -> Result<(), TemplateApplicationError<ContentProcessor::Error>> {
    let markup5ever_rcdom::NodeData::Text { ref contents } = handle.data else {
        return Err(TemplateApplicationError::TemplateNonTextContent);
    };

    let mut contents = contents.borrow_mut();

    let new_contents = content_processor
        .process(&contents)
        .map_err(TemplateApplicationError::ContentProcessor)?;

    *contents = new_contents.into();

    Ok(())
}
