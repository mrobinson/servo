/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use itertools::Itertools;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use style::str::split_commas;

use crate::dom::bindings::str::{DOMString, FromInputValueString};
use crate::dom::htmlinputelement::inputtype::SpecificInputType;
use crate::dom::types::HTMLInputElement;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct EmailInputType();

impl SpecificInputType for EmailInputType {
    fn sanitize_value(&self, input: &HTMLInputElement, value: &mut DOMString) {
        if !input.Multiple() {
            value.strip_newlines();
            value.strip_leading_and_trailing_ascii_whitespace();
        } else {
            let sanitized = split_commas(&value.str())
                .map(|token| {
                    let mut token = DOMString::from(token.to_string());
                    token.strip_newlines();
                    token.strip_leading_and_trailing_ascii_whitespace();
                    token
                })
                .join(",");
            value.clear();
            value.push_str(sanitized.as_str());
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#email-state-(type=email):suffering-from-bad-input>
    /// <https://html.spec.whatwg.org/multipage/i#email-state-(type=email):suffering-from-bad-input-2>
    fn suffers_from_bad_input(&self, _value: &DOMString) -> bool {
        // TODO: Check for input that cannot be converted to punycode.
        // Currently we don't support conversion of email values to punycode
        // so always return false.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#e-mail-state-(type=email):suffering-from-a-type-mismatch>
    /// <https://html.spec.whatwg.org/multipage/#e-mail-state-(type=email):suffering-from-a-type-mismatch-2>
    fn suffers_from_type_mismatch(&self, input: &HTMLInputElement, value: &DOMString) -> bool {
        if input.Multiple() {
            !split_commas(&value.str()).all(|string| string.is_valid_email_address_string())
        } else {
            !value.str().is_valid_email_address_string()
        }
    }
}
