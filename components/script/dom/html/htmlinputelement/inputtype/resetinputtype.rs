/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use script_bindings::domstring::DOMString;
use script_bindings::script_runtime::CanGc;

use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlformelement::{FormControl, ResetFrom};
use crate::dom::htmlinputelement::{HTMLInputElement};
use crate::dom::htmlinputelement::inputtype::SpecificInputType;
use crate::dom::node::NodeTraits;

const DEFAULT_RESET_VALUE: &str = "Reset";

pub(crate) struct ResetInputType();

impl SpecificInputType for ResetInputType {
    fn value_for_shadow_dom(&self, _input: &HTMLInputElement) -> DOMString {
        DEFAULT_RESET_VALUE.into()
    }

    /// <https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        can_gc: CanGc,
    ) {
        // Step 1: If the element does not have a form owner, then return.
        if let Some(form_owner) = input.form_owner() {
            let document = input.owner_document();

            // Step 2: If the element's node document is not fully active, then return.
            if !document.is_fully_active() {
                return;
            }

            // Step 3: Reset the form owner from the element.
            form_owner.reset(ResetFrom::NotFromForm, can_gc);
        }
    }
}
