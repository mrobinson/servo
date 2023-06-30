/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Enumerate;
use std::str::Chars;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::virtualmethods::VirtualMethods;
use super::node::{Node, BindContext, UnbindContext, window_from_node};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;
use script_traits::{UserZoom, ViewportConstraints};
use servo_config::pref;
use style::str::HTML_SPACE_CHARACTERS;
use style_traits::PinchZoomFactor;

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLMetaElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "viewport" {
                self.apply_viewport();
            }

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    fn apply_viewport(&self) {
        if !pref!(layout.viewport.enabled) {
            return;
        }

        let element = self.upcast::<Element>();
        if let Some(content) = element.get_attribute(&ns!(), &local_name!("content")) {
            let content = content.value();
            if content.is_empty() {
                return;
            }

            if let Some(constraints) = ViewportConstraints::from_meta(&**content) {
                window_from_node(self).set_viewport_constraints(constraints);
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    fn apply_referrer(&self) {
        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            if let Some(head) = parent.downcast::<HTMLHeadElement>() {
                head.set_document_referrer();
            }
        }
    }
}

impl HTMLMetaElementMethods for HTMLMetaElement {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");
}

impl VirtualMethods for HTMLMetaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected {
            self.process_attributes();
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }

        self.process_referrer_attribute();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            self.process_referrer_attribute();
        }
    }
}


trait FromMeta: Sized {
    fn from_meta(value: &str) -> Option<Self>;
}

impl FromMeta for f32 {
    fn from_meta(value: &str) -> Option<Self> {
        match value {
            v if v.eq_ignore_ascii_case("device-width") => None,
            v if v.eq_ignore_ascii_case("device-height") => None,
            _ => match value.parse::<f32>() {
                Ok(n) if n >= 0. => Some(n.max(1.).min(10000.)),
                Ok(_) => return None,
                Err(_) => Some(1.),
            },
        }
    }
}

impl FromMeta for PinchZoomFactor {
    fn from_meta(value: &str) -> Option<Self> {
        Some(match value {
            v if v.eq_ignore_ascii_case("yes") => PinchZoomFactor::new(1.),
            v if v.eq_ignore_ascii_case("no") => PinchZoomFactor::new(0.1),
            v if v.eq_ignore_ascii_case("device-width") => PinchZoomFactor::new(10.),
            v if v.eq_ignore_ascii_case("device-height") => PinchZoomFactor::new(10.),
            _ => match value.parse::<f32>() {
                Ok(n) if n >= 0. => PinchZoomFactor::new(n.max(0.1).min(10.)),
                Ok(_) => return None,
                Err(_) => PinchZoomFactor::new(0.1),
            },
        })
    }
}

impl FromMeta for UserZoom {
    fn from_meta(value: &str) -> Option<UserZoom> {
        Some(match value {
            v if v.eq_ignore_ascii_case("yes") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("no") => UserZoom::Fixed,
            v if v.eq_ignore_ascii_case("device-width") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("device-height") => UserZoom::Zoom,
            _ => match value.parse::<f32>() {
                Ok(n) if n >= 1. || n <= -1. => UserZoom::Zoom,
                _ => UserZoom::Fixed,
            },
        })
    }
}

/// Whitespace as defined by DEVICE-ADAPT § 9.2
// TODO: should we just use whitespace as defined by HTML5?
const WHITESPACE: &'static [char] = &['\t', '\n', '\r', ' '];

/// Separators as defined by DEVICE-ADAPT § 9.2
// need to use \x2c instead of ',' due to test-tidy
const SEPARATOR: &'static [char] = &['\x2c', ';'];

#[inline]
fn is_whitespace_separator_or_equals(c: &char) -> bool {
    WHITESPACE.contains(c) || SEPARATOR.contains(c) || *c == '='
}

impl FromMeta for ViewportConstraints {
    #[allow(missing_docs)]
    fn from_meta(content: &str) -> Option<Self> {
        let mut width = None;
        let mut height = None;
        let mut initial_scale = None;
        let mut min_scale = None;
        let mut max_scale = None;
        let mut user_zoom: Option<UserZoom> = None;

        let mut iter = content.chars().enumerate();

        macro_rules! start_of_name {
            ($iter:ident) => {
                $iter
                    .by_ref()
                    .skip_while(|&(_, c)| is_whitespace_separator_or_equals(&c))
                    .next()
            };
        }

        while let Some((start, _)) = start_of_name!(iter) {
            let property = parse_viewport_meta_property(content, &mut iter, start);

            if let Some((name, value)) = property {
                match name {
                    n if n.eq_ignore_ascii_case("width") => {
                        width = f32::from_meta(value);
                    },
                    n if n.eq_ignore_ascii_case("height") => {
                        height = f32::from_meta(value);
                    },
                    n if n.eq_ignore_ascii_case("initial-scale") => {
                        initial_scale = PinchZoomFactor::from_meta(value);
                    },
                    n if n.eq_ignore_ascii_case("minimum-scale") => {
                        min_scale = PinchZoomFactor::from_meta(value);
                    }
                    n if n.eq_ignore_ascii_case("maximum-scale") => {
                        max_scale = PinchZoomFactor::from_meta(value);
                    }
                    n if n.eq_ignore_ascii_case("user-scalable") => {
                        user_zoom = UserZoom::from_meta(value);
                    },
                    _ => {},
                }
            }
        }

        Some(ViewportConstraints{
            width,
            height,
            initial_scale: initial_scale.unwrap_or(PinchZoomFactor::new(1.)),
            min_scale,
            max_scale,
            user_zoom: user_zoom.unwrap_or(UserZoom::Zoom),
        })
    }
}

fn parse_viewport_meta_property<'a>(
    content: &'a str,
    iter: &mut Enumerate<Chars<'a>>,
    start: usize,
) -> Option<(&'a str, &'a str)> {
    fn end_of_token(iter: &mut Enumerate<Chars>) -> Option<(usize, char)> {
        iter.by_ref()
            .skip_while(|&(_, c)| !is_whitespace_separator_or_equals(&c))
            .next()
    }

    fn skip_whitespace(iter: &mut Enumerate<Chars>) -> Option<(usize, char)> {
        iter.by_ref()
            .skip_while(|&(_, c)| WHITESPACE.contains(&c))
            .next()
    }

    // <name> <whitespace>* '='
    let end = match end_of_token(iter) {
        Some((end, c)) if WHITESPACE.contains(&c) => match skip_whitespace(iter) {
            Some((_, c)) if c == '=' => end,
            _ => return None,
        },
        Some((end, c)) if c == '=' => end,
        _ => return None,
    };
    let name = &content[start..end];

    // <whitespace>* <value>
    let start = match skip_whitespace(iter) {
        Some((start, c)) if !SEPARATOR.contains(&c) => start,
        _ => return None,
    };
    let value = match end_of_token(iter) {
        Some((end, _)) => &content[start..end],
        _ => &content[start..],
    };

    Some((name, value))
}