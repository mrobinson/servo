/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use stylo_atoms::Atom;
use time::OffsetDateTime;
use embedder_traits::InputMethodType;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::DOMString;
use script_bindings::script_runtime::CanGc;
use crate::dom::attr::Attr;
use crate::dom::element::AttributeMutation;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::{HTMLInputElement, InputActivationState, ValueMode};
use crate::dom::inputtype::buttoninputtype::ButtonInputType;
use crate::dom::inputtype::checkboxinputtype::CheckboxInputType;
use crate::dom::inputtype::colorinputtype::ColorInputType;
use crate::dom::inputtype::dateinputtype::DateInputType;
use crate::dom::inputtype::datetimelocalinputtype::DatetimeLocalInputType;
use crate::dom::inputtype::emailinputtype::EmailInputType;
use crate::dom::inputtype::fileinputtype::FileInputType;
use crate::dom::inputtype::hiddeninputtype::HiddenInputType;
use crate::dom::inputtype::imageinputtype::ImageInputType;
use crate::dom::inputtype::monthinputtype::MonthInputType;
use crate::dom::inputtype::numberinputtype::NumberInputType;
use crate::dom::inputtype::passwordinputtype::PasswordInputType;
use crate::dom::inputtype::radioinputtype::RadioInputType;
use crate::dom::inputtype::rangeinputtype::RangeInputType;
use crate::dom::inputtype::resetinputtype::ResetInputType;
use crate::dom::inputtype::searchinputtype::SearchInputType;
use crate::dom::inputtype::submitinputtype::SubmitInputType;
use crate::dom::inputtype::telinputtype::TelInputType;
use crate::dom::inputtype::textinputtype::TextInputType;
use crate::dom::inputtype::timeinputtype::TimeInputType;
use crate::dom::inputtype::urlinputtype::UrlInputType;
use crate::dom::inputtype::weekinputtype::WeekInputType;
use crate::dom::node::{BindContext, UnbindContext};

pub(crate) mod buttoninputtype;
pub(crate) mod checkboxinputtype;
pub(crate) mod colorinputtype;
pub(crate) mod dateinputtype;
pub(crate) mod datetimelocalinputtype;
pub(crate) mod emailinputtype;
pub(crate) mod fileinputtype;
pub(crate) mod hiddeninputtype;
pub(crate) mod imageinputtype;
pub(crate) mod monthinputtype;
pub(crate) mod numberinputtype;
pub(crate) mod passwordinputtype;
pub(crate) mod radioinputtype;
pub(crate) mod rangeinputtype;
pub(crate) mod resetinputtype;
pub(crate) mod searchinputtype;
pub(crate) mod submitinputtype;
pub(crate) mod telinputtype;
pub(crate) mod textinputtype;
pub(crate) mod timeinputtype;
pub(crate) mod urlinputtype;
pub(crate) mod weekinputtype;

/// <https://html.spec.whatwg.org/multipage/#attr-input-type>
#[derive(Clone, Copy, Debug, Default, JSTraceable, PartialEq, MallocSizeOf)]
pub(crate) enum InputType {
    /// <https://html.spec.whatwg.org/multipage/#button-state-(type=button)>
    Button,

    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox)>
    Checkbox,

    /// <https://html.spec.whatwg.org/multipage/#color-state-(type=color)>
    Color,

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date)>
    Date,

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local)>
    DatetimeLocal,

    /// <https://html.spec.whatwg.org/multipage/#email-state-(type=email)>
    Email,

    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file)>
    File,

    /// <https://html.spec.whatwg.org/multipage/#hidden-state-(type=hidden)>
    Hidden,

    /// <https://html.spec.whatwg.org/multipage/#image-button-state-(type=image)>
    Image,

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month)>
    Month,

    /// <https://html.spec.whatwg.org/multipage/#number-state-(type=number)>
    Number,

    /// <https://html.spec.whatwg.org/multipage/#password-state-(type=password)>
    Password,

    /// <https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio)>
    Radio,

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range)>
    Range,

    /// <https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset)>
    Reset,

    /// <https://html.spec.whatwg.org/multipage/#text-(type=text)-state-and-search-state-(type=search)>
    Search,

    /// <https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit)>
    Submit,

    /// <https://html.spec.whatwg.org/multipage/#telephone-state-(type=tel)>
    Tel,

    /// <https://html.spec.whatwg.org/multipage/#text-(type=text)-state-and-search-state-(type=search)>
    #[default]
    Text,

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time)>
    Time,

    /// <https://html.spec.whatwg.org/multipage/#url-state-(type=url)>
    Url,

    /// <https://html.spec.whatwg.org/multipage/#week-state-(type=week)>
    Week,
}

impl InputType {
    pub(crate) fn as_specific(&self) -> &dyn SpecificInputType {
        match *self {
            Self::Button => &ButtonInputType() as &dyn SpecificInputType,
            Self::Checkbox => &CheckboxInputType() as &dyn SpecificInputType,
            Self::Color => &ColorInputType() as &dyn SpecificInputType,
            Self::Date => &DateInputType() as &dyn SpecificInputType,
            Self::DatetimeLocal => &DatetimeLocalInputType() as &dyn SpecificInputType,
            Self::Email => &EmailInputType() as &dyn SpecificInputType,
            Self::File => &FileInputType() as &dyn SpecificInputType,
            Self::Hidden => &HiddenInputType() as &dyn SpecificInputType,
            Self::Image => &ImageInputType() as &dyn SpecificInputType,
            Self::Month => &MonthInputType() as &dyn SpecificInputType,
            Self::Number => &NumberInputType() as &dyn SpecificInputType,
            Self::Password => &PasswordInputType() as &dyn SpecificInputType,
            Self::Radio => &RadioInputType() as &dyn SpecificInputType,
            Self::Range => &RangeInputType() as &dyn SpecificInputType,
            Self::Reset => &ResetInputType() as &dyn SpecificInputType,
            Self::Search => &SearchInputType() as &dyn SpecificInputType,
            Self::Submit => &SubmitInputType() as &dyn SpecificInputType,
            Self::Tel => &TelInputType() as &dyn SpecificInputType,
            Self::Text => &TextInputType() as &dyn SpecificInputType,
            Self::Time => &TimeInputType() as &dyn SpecificInputType,
            Self::Url => &UrlInputType() as &dyn SpecificInputType,
            Self::Week => &WeekInputType() as &dyn SpecificInputType,
        }
    }

    /// Defines which input type that should perform like a text input,
    /// specifically when it is interacting with JS. Note that Password
    /// is not included here since it is handled slightly differently,
    /// with placeholder characters shown rather than the underlying value.
    pub(crate) fn is_textual(&self) -> bool {
        matches!(
            *self,
            InputType::Date |
                InputType::DatetimeLocal |
                InputType::Email |
                InputType::Hidden |
                InputType::Month |
                InputType::Number |
                InputType::Range |
                InputType::Search |
                InputType::Tel |
                InputType::Text |
                InputType::Time |
                InputType::Url |
                InputType::Week
        )
    }

    pub(crate) fn is_textual_or_password(&self) -> bool {
        self.is_textual() || *self == InputType::Password
    }

    /// <https://html.spec.whatwg.org/multipage/#has-a-periodic-domain>
    pub(crate) fn has_periodic_domain(&self) -> bool {
        *self == InputType::Time
    }

    pub(crate) fn as_str(&self) -> &str {
        match *self {
            InputType::Button => "button",
            InputType::Checkbox => "checkbox",
            InputType::Color => "color",
            InputType::Date => "date",
            InputType::DatetimeLocal => "datetime-local",
            InputType::Email => "email",
            InputType::File => "file",
            InputType::Hidden => "hidden",
            InputType::Image => "image",
            InputType::Month => "month",
            InputType::Number => "number",
            InputType::Password => "password",
            InputType::Radio => "radio",
            InputType::Range => "range",
            InputType::Reset => "reset",
            InputType::Search => "search",
            InputType::Submit => "submit",
            InputType::Tel => "tel",
            InputType::Text => "text",
            InputType::Time => "time",
            InputType::Url => "url",
            InputType::Week => "week",
        }
    }
}

impl TryFrom<InputType> for InputMethodType {
    type Error = &'static str;

    fn try_from(input_type: InputType) -> Result<Self, Self::Error> {
        match input_type {
            InputType::Color => Ok(InputMethodType::Color),
            InputType::Date => Ok(InputMethodType::Date),
            InputType::DatetimeLocal => Ok(InputMethodType::DatetimeLocal),
            InputType::Email => Ok(InputMethodType::Email),
            InputType::Month => Ok(InputMethodType::Month),
            InputType::Number => Ok(InputMethodType::Number),
            InputType::Password => Ok(InputMethodType::Password),
            InputType::Search => Ok(InputMethodType::Search),
            InputType::Tel => Ok(InputMethodType::Tel),
            InputType::Text => Ok(InputMethodType::Text),
            InputType::Time => Ok(InputMethodType::Time),
            InputType::Url => Ok(InputMethodType::Url),
            InputType::Week => Ok(InputMethodType::Week),
            _ => Err("Input does not support IME."),
        }
    }
}

impl From<&Atom> for InputType {
    fn from(value: &Atom) -> InputType {
        match value.to_ascii_lowercase() {
            atom!("button") => InputType::Button,
            atom!("checkbox") => InputType::Checkbox,
            atom!("color") => InputType::Color,
            atom!("date") => InputType::Date,
            atom!("datetime-local") => InputType::DatetimeLocal,
            atom!("email") => InputType::Email,
            atom!("file") => InputType::File,
            atom!("hidden") => InputType::Hidden,
            atom!("image") => InputType::Image,
            atom!("month") => InputType::Month,
            atom!("number") => InputType::Number,
            atom!("password") => InputType::Password,
            atom!("radio") => InputType::Radio,
            atom!("range") => InputType::Range,
            atom!("reset") => InputType::Reset,
            atom!("search") => InputType::Search,
            atom!("submit") => InputType::Submit,
            atom!("tel") => InputType::Tel,
            atom!("text") => InputType::Text,
            atom!("time") => InputType::Time,
            atom!("url") => InputType::Url,
            atom!("week") => InputType::Week,
            _ => Self::default(),
        }
    }
}

pub(crate) trait SpecificInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, _value: &mut DOMString) {}

    fn convert_string_to_number(&self, _value: &str) -> Option<f64> {
        None
    }

    fn convert_number_to_string(&self, _value: f64) -> Option<DOMString> {
        unreachable!("Should not have called convert_number_to_string for non-Date types")
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-input-value-string-date>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, _value: DOMString) -> Option<OffsetDateTime> {
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-input-value-date-string>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in SetValueAsDate
    fn convert_datetime_to_dom_string(&self, _value: OffsetDateTime) -> DOMString {
        unreachable!("Should not have called convert_datetime_to_string for non-Date types")
    }

    /// <https://html.spec.whatwg.org/multipage/#the-required-attribute%3Asuffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, value: &DOMString) -> bool {
        input.Required() &&
            input.value_mode() == ValueMode::Value &&
            input.is_mutable() &&
            value.is_empty()
    }

    fn suffers_from_bad_input(&self, _value: &DOMString) -> bool {
        false
    }

    fn suffers_from_type_mismatch(&self, _input: &HTMLInputElement, _value: &DOMString) -> bool {
        false
    }

    fn value_for_shadow_dom(&self, _input: &HTMLInputElement) -> DOMString {
        "".into()
    }

    /// <https://html.spec.whatwg.org/multipage/#signal-a-type-change>
    fn signal_type_change(&self, _input: &HTMLInputElement, _can_gc: CanGc) {}

    fn activation_behavior(
        &self,
        _input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        _can_gc: CanGc,
    ) {
    }

    fn legacy_pre_activation_behavior(
        &self,
        _input: &HTMLInputElement,
        _can_gc: CanGc,
    ) -> Option<InputActivationState> {
        None
    }

    fn legacy_canceled_activation_behavior(
        &self,
        _input: &HTMLInputElement,
        _cache: InputActivationState,
        _can_gc: CanGc,
    ) {
    }

    fn show_the_picker_if_applicable(&self, _input: &HTMLInputElement) {}

    fn select_files(&self, _input: &HTMLInputElement, _test_paths: Option<Vec<DOMString>>) {}

    fn attribute_mutated(
        &self,
        _input: &HTMLInputElement,
        _attr: &Attr,
        _mutation: AttributeMutation,
        _can_gc: CanGc,
    ) {
    }

    fn bind_to_tree(&self, _input: &HTMLInputElement, _cx: &mut JSContext, _context: &BindContext) {
    }

    fn unbind_from_tree(
        &self,
        _input: &HTMLInputElement,
        _context: &UnbindContext,
        _can_gc: CanGc,
    ) {
    }
}