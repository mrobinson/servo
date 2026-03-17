/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::PathBuf;
use std::str::FromStr;

use embedder_traits::{EmbedderControlRequest, FilePickerRequest, FilterPattern};
use script_bindings::codegen::GenericBindings::FileListBinding::FileListMethods;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::DOMString;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use style::str::split_commas;

use crate::dom::document_embedder_controls::ControlElement;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::{HTMLInputElement};
use crate::dom::htmlinputelement::inputtype::SpecificInputType;
use crate::dom::node::NodeTraits;

const DEFAULT_FILE_INPUT_VALUE: &str = "No file chosen";
const DEFAULT_FILE_INPUT_MULTIPLE_VALUE: &str = "No files chosen";

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct FileInputType();

impl SpecificInputType for FileInputType {
    fn value_for_shadow_dom(&self, input: &HTMLInputElement) -> DOMString {
        let Some(filelist) = input.filelist() else {
            if input.Multiple() {
                return DEFAULT_FILE_INPUT_MULTIPLE_VALUE.into();
            }
            return DEFAULT_FILE_INPUT_VALUE.into();
        };
        let length = filelist.Length();
        if length > 1 {
            return format!("{length} files").into();
        }

        let Some(first_item) = filelist.Item(0) else {
            if input.Multiple() {
                return DEFAULT_FILE_INPUT_MULTIPLE_VALUE.into();
            }
            return DEFAULT_FILE_INPUT_VALUE.into();
        };
        first_item.name().to_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file):suffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, _value: &DOMString) -> bool {
        input.Required() && input.filelist().is_none_or(|files| files.Length() == 0)
    }

    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        _can_gc: CanGc,
    ) {
        input.show_the_picker_if_applicable();
    }

    fn show_the_picker_if_applicable(&self, input: &HTMLInputElement) {
        self.select_files(input, None)
    }

    /// Select files by invoking UI or by passed in argument.
    ///
    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file)>
    fn select_files(&self, input: &HTMLInputElement, test_paths: Option<Vec<DOMString>>) {
        let current_paths = match &test_paths {
            Some(test_paths) => test_paths
                .iter()
                .filter_map(|path_str| PathBuf::from_str(&path_str.str()).ok())
                .collect(),
            // TODO: This should get the pathnames of the current files, but we currently don't have
            // that information in Script. It should be passed through here.
            None => Default::default(),
        };

        let accept_current_paths_for_testing = test_paths.is_some();
        input
            .owner_document()
            .embedder_controls()
            .show_embedder_control(
                ControlElement::FileInput(DomRoot::from_ref(input)),
                EmbedderControlRequest::FilePicker(FilePickerRequest {
                    origin: input.owner_window().origin().immutable().clone(),
                    current_paths,
                    filter_patterns: filter_from_accept(&input.Accept()),
                    allow_select_multiple: input.Multiple(),
                    accept_current_paths_for_testing,
                }),
                None,
            );
    }
}

/// <https://html.spec.whatwg.org/multipage/#attr-input-accept>
fn filter_from_accept(s: &DOMString) -> Vec<FilterPattern> {
    let mut filter = vec![];
    for p in split_commas(&s.str()) {
        let p = p.trim();
        if let Some('.') = p.chars().next() {
            filter.push(FilterPattern(p[1..].to_string()));
        } else if let Some(exts) = mime_guess::get_mime_extensions_str(p) {
            for ext in exts {
                filter.push(FilterPattern(ext.to_string()));
            }
        }
    }

    filter
}
