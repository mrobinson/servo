/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use script_bindings::script_runtime::CanGc;

use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlformelement::{FormControl, FormSubmitterElement, SubmittedFrom};
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmlinputelement::inputtype::SpecificInputType;
use crate::dom::node::NodeTraits;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct ImageInputType();

impl SpecificInputType for ImageInputType {
    /// <https://html.spec.whatwg.org/multipage/#image-button-state-(type=image):input-activation-behavior>
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

            // TODO Step 3. If the user activated the control while explicitly selecting a coordinate,
            // then set the element's selected coordinate to that coordinate.

            // Step 4: Submit the element's form owner from the element with userInvolvement
            // set to event's user navigation involvement.
            form_owner.submit(
                SubmittedFrom::NotFromForm,
                FormSubmitterElement::Input(input),
                can_gc,
            )
        }
    }
}
