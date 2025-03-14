use html5ever::interface::TreeSink;

/// The handle type for the DOM nodes.
pub type Handle = <markup5ever_rcdom::RcDom as TreeSink>::Handle;

/// A struct representing the template node lookup.
pub struct TemplateNodeLookup<TemplateElementFilter> {
    /// The RcDom instance.
    pub rcdom: markup5ever_rcdom::RcDom,
    /// The filter to select the template element.
    pub template_element_filter: TemplateElementFilter,
    /// The template element handle.
    pub template_element: std::cell::RefCell<Option<Handle>>,
}

/// A trait for filtering template elements.
pub trait TemplateElementFilter {
    /// Check if the element is selected.
    fn is_selected(
        &self,
        name: &html5ever::QualName,
        attrs: &[html5ever::Attribute],
        flags: &html5ever::interface::ElementFlags,
    ) -> bool;
}

impl<T: TemplateElementFilter> TemplateElementFilter for &T {
    fn is_selected(
        &self,
        name: &html5ever::QualName,
        attrs: &[html5ever::Attribute],
        flags: &html5ever::interface::ElementFlags,
    ) -> bool {
        (*self).is_selected(name, attrs, flags)
    }
}

/// Module containing template element filters.
pub mod template_element_filter {
    /// A filter for script tags with a specific type.
    pub struct ScriptTag {
        /// The script type to filter.
        pub script_type: std::borrow::Cow<'static, str>,
    }

    impl super::TemplateElementFilter for ScriptTag {
        fn is_selected(
            &self,
            name: &html5ever::QualName,
            attrs: &[html5ever::Attribute],
            _flags: &html5ever::interface::ElementFlags,
        ) -> bool {
            name.local == html5ever::local_name!("script")
                && attrs.iter().any(|attr| {
                    attr.name.local == html5ever::local_name!("type")
                        && self.script_type == *attr.value
                })
        }
    }
}

impl<TemplateElementFilter> TemplateNodeLookup<TemplateElementFilter> {
    /// Create a new TemplateNodeLookup instance.
    pub fn new(template_element_filter: TemplateElementFilter) -> Self {
        Self {
            rcdom: Default::default(),
            template_element_filter,
            template_element: std::cell::RefCell::new(None),
        }
    }
}

impl<TemplateElementFilter: self::TemplateElementFilter> TreeSink
    for TemplateNodeLookup<TemplateElementFilter>
{
    type Handle = Handle;

    type Output = Self;

    type ElemName<'a> = <markup5ever_rcdom::RcDom as TreeSink>::ElemName<'a>
    where
        TemplateElementFilter: 'a;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        self.rcdom.parse_error(msg);
    }

    fn get_document(&self) -> Self::Handle {
        self.rcdom.get_document()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        self.rcdom.elem_name(target)
    }

    fn create_element(
        &self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        let is_target_element = self
            .template_element_filter
            .is_selected(&name, &attrs, &flags);

        let handle = self.rcdom.create_element(name, attrs, flags);
        if is_target_element {
            let mut template_element = self.template_element.borrow_mut();
            if template_element.is_none() {
                *template_element = Some(std::rc::Rc::clone(&handle));
            }
        }
        handle
    }

    fn create_comment(&self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.rcdom.create_comment(text)
    }

    fn create_pi(
        &self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        self.rcdom.create_pi(target, data)
    }

    fn append(&self, parent: &Self::Handle, child: html5ever::interface::NodeOrText<Self::Handle>) {
        self.rcdom.append(parent, child);
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        self.rcdom
            .append_based_on_parent_node(element, prev_element, child);
    }

    fn append_doctype_to_document(
        &self,
        name: html5ever::tendril::StrTendril,
        public_id: html5ever::tendril::StrTendril,
        system_id: html5ever::tendril::StrTendril,
    ) {
        self.rcdom
            .append_doctype_to_document(name, public_id, system_id);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.rcdom.get_template_contents(target)
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        self.rcdom.same_node(x, y)
    }

    fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
        self.rcdom.set_quirks_mode(mode);
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        self.rcdom.append_before_sibling(sibling, new_node);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        self.rcdom.add_attrs_if_missing(target, attrs);
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.rcdom.remove_from_parent(target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        self.rcdom.reparent_children(node, new_parent);
    }
}
