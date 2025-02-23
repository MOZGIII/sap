//! The HTML templating logic.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

mod dom;

use html5ever::tendril::TendrilSink as _;
use markup5ever_rcdom as rcdom;
use std::borrow::Cow;

pub use dom::{template_element_filter, TemplateElementFilter};

/// The HTML teplating processor.
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

    #[error("template script content not found")]
    TemplateContentNotFound,

    #[error("template element has more than one child")]
    TemplateElementHasMoreThanOneChild,

    #[error("template application: {0}")]
    TemplateApplication(TemplateApplicationError<ContentProcessorError>),
}

/// An abstract content processor.
pub trait ContentProcessor {
    /// An error that could be encountered while processing the content.
    type Error;

    /// Process the provided content and get replacement content.
    fn process(&self, input: &str) -> Result<String, Self::Error>;
}

impl<T, E> ContentProcessor for T
where
    T: Fn(&str) -> Result<String, E>,
{
    type Error = E;

    fn process(&self, input: &str) -> Result<String, Self::Error> {
        (self)(input)
    }
}

impl ContentProcessor for () {
    type Error = std::convert::Infallible;

    fn process(&self, input: &str) -> Result<String, Self::Error> {
        Ok(input.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateApplicationError<ContentProcessorError> {
    #[error("no text content")]
    TemplateNonTextContent,

    #[error("content processor error: {0}")]
    ContentProcessor(ContentProcessorError),
}

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

impl<TemplateElementFilter, ContentProcessor> Processor<TemplateElementFilter, ContentProcessor>
where
    TemplateElementFilter: self::dom::TemplateElementFilter,
    ContentProcessor: self::ContentProcessor,
{
    /// Process the HTML template.
    pub fn process(
        &self,
        html: &[u8],
    ) -> Result<Vec<u8>, TemplatingError<<ContentProcessor as self::ContentProcessor>::Error>> {
        let parser = html5ever::parse_document(
            dom::TemplateNodeLookup::new(&self.template_element_filter),
            Default::default(),
        );

        let dom = parser.from_utf8().one(html);

        let dom::TemplateNodeLookup {
            rcdom,
            template_element,
            ..
        } = dom;

        let rcdom::RcDom {
            document, errors, ..
        } = rcdom;

        let errors = errors.into_inner();
        if !errors.is_empty() {
            tracing::warn!(message = "parsing errors", ?errors);
        }

        let template_element = template_element.into_inner();

        let Some(template_element) = template_element else {
            return Err(TemplatingError::TemplateNotFound);
        };

        let children = template_element.children.borrow();
        if children.len() > 1 {
            return Err(TemplatingError::TemplateElementHasMoreThanOneChild);
        }

        let Some(child) = children.first() else {
            return Err(TemplatingError::TemplateContentNotFound);
        };

        let child = std::rc::Rc::clone(child);
        drop(children);

        apply_template(&child, &self.content_processor)
            .map_err(TemplatingError::TemplateApplication)?;

        let mut output = Vec::with_capacity(html.len());

        let serializable_document: rcdom::SerializableHandle = document.into();

        html5ever::serialize::serialize(
            &mut output,
            &serializable_document,
            html5ever::serialize::SerializeOpts {
                scripting_enabled: true,
                traversal_scope: html5ever::serialize::TraversalScope::ChildrenOnly(None),
                create_missing_parent: false,
            },
        )
        .unwrap(); // vec write never fails

        Ok(output)
    }
}
