/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use embedder_traits::InputMethodType;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::DOMString;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use stylo_atoms::Atom;
use time::OffsetDateTime;

use crate::dom::attr::Attr;
use crate::dom::element::AttributeMutation;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::filelist::FileList;
use crate::dom::input_element::button_input_type::ButtonInputType;
use crate::dom::input_element::checkbox_input_type::CheckboxInputType;
use crate::dom::input_element::color_input_type::ColorInputType;
use crate::dom::input_element::date_input_type::DateInputType;
use crate::dom::input_element::datetime_local_input_type::DatetimeLocalInputType;
use crate::dom::input_element::email_input_type::EmailInputType;
use crate::dom::input_element::file_input_type::FileInputType;
use crate::dom::input_element::hidden_input_type::HiddenInputType;
use crate::dom::input_element::image_input_type::ImageInputType;
use crate::dom::input_element::month_input_type::MonthInputType;
use crate::dom::input_element::number_input_type::NumberInputType;
use crate::dom::input_element::password_input_type::PasswordInputType;
use crate::dom::input_element::radio_input_type::RadioInputType;
use crate::dom::input_element::range_input_type::RangeInputType;
use crate::dom::input_element::reset_input_type::ResetInputType;
use crate::dom::input_element::search_input_type::SearchInputType;
use crate::dom::input_element::submit_input_type::SubmitInputType;
use crate::dom::input_element::tel_input_type::TelInputType;
use crate::dom::input_element::text_input_type::TextInputType;
use crate::dom::input_element::time_input_type::TimeInputType;
use crate::dom::input_element::url_input_type::UrlInputType;
use crate::dom::input_element::week_input_type::WeekInputType;
use crate::dom::input_element::{HTMLInputElement, InputActivationState, ValueMode};
use crate::dom::node::{BindContext, UnbindContext};

/// <https://html.spec.whatwg.org/multipage/#attr-input-type>
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum InputType {
    /// <https://html.spec.whatwg.org/multipage/#button-state-(type=button)>
    Button(ButtonInputType),

    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox)>
    Checkbox(CheckboxInputType),

    /// <https://html.spec.whatwg.org/multipage/#color-state-(type=color)>
    Color(ColorInputType),

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date)>
    Date(DateInputType),

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local)>
    DatetimeLocal(DatetimeLocalInputType),

    /// <https://html.spec.whatwg.org/multipage/#email-state-(type=email)>
    Email(EmailInputType),

    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file)>
    File(FileInputType),

    /// <https://html.spec.whatwg.org/multipage/#hidden-state-(type=hidden)>
    Hidden(HiddenInputType),

    /// <https://html.spec.whatwg.org/multipage/#image-button-state-(type=image)>
    Image(ImageInputType),

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month)>
    Month(MonthInputType),

    /// <https://html.spec.whatwg.org/multipage/#number-state-(type=number)>
    Number(NumberInputType),

    /// <https://html.spec.whatwg.org/multipage/#password-state-(type=password)>
    Password(PasswordInputType),

    /// <https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio)>
    Radio(RadioInputType),

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range)>
    Range(RangeInputType),

    /// <https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset)>
    Reset(ResetInputType),

    /// <https://html.spec.whatwg.org/multipage/#text-(type=text)-state-and-search-state-(type=search)>
    Search(SearchInputType),

    /// <https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit)>
    Submit(SubmitInputType),

    /// <https://html.spec.whatwg.org/multipage/#telephone-state-(type=tel)>
    Tel(TelInputType),

    /// <https://html.spec.whatwg.org/multipage/#text-(type=text)-state-and-search-state-(type=search)>
    Text(TextInputType),

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time)>
    Time(TimeInputType),

    /// <https://html.spec.whatwg.org/multipage/#url-state-(type=url)>
    Url(UrlInputType),

    /// <https://html.spec.whatwg.org/multipage/#week-state-(type=week)>
    Week(WeekInputType),
}

impl InputType {
    pub fn button() -> Self {
        Self::Button(ButtonInputType::default())
    }

    pub fn checkbox() -> Self {
        Self::Checkbox(CheckboxInputType::default())
    }

    pub fn color() -> Self {
        Self::Color(ColorInputType::default())
    }

    pub fn date() -> Self {
        Self::Date(DateInputType::default())
    }

    pub fn datetime_local() -> Self {
        Self::DatetimeLocal(DatetimeLocalInputType::default())
    }

    pub fn email() -> Self {
        Self::Email(EmailInputType::default())
    }

    pub fn file() -> Self {
        Self::File(FileInputType::default())
    }

    pub fn hidden() -> Self {
        Self::Hidden(HiddenInputType::default())
    }

    pub fn image() -> Self {
        Self::Image(ImageInputType::default())
    }

    pub fn month() -> Self {
        Self::Month(MonthInputType::default())
    }

    pub fn number() -> Self {
        Self::Number(NumberInputType::default())
    }

    pub fn password() -> Self {
        Self::Password(PasswordInputType::default())
    }

    pub fn radio() -> Self {
        Self::Radio(RadioInputType::default())
    }

    pub fn range() -> Self {
        Self::Range(RangeInputType::default())
    }

    pub fn reset() -> Self {
        Self::Reset(ResetInputType::default())
    }

    pub fn search() -> Self {
        Self::Search(SearchInputType::default())
    }

    pub fn submit() -> Self {
        Self::Submit(SubmitInputType::default())
    }

    pub fn tel() -> Self {
        Self::Tel(TelInputType::default())
    }

    pub fn text() -> Self {
        Self::Text(TextInputType::default())
    }

    pub fn time() -> Self {
        Self::Time(TimeInputType::default())
    }

    pub fn url() -> Self {
        Self::Url(UrlInputType::default())
    }

    pub fn week() -> Self {
        Self::Week(WeekInputType::default())
    }

    pub(crate) fn as_specific(&self) -> &dyn SpecificInputType {
        match self {
            Self::Button(input_type) => input_type as &dyn SpecificInputType,
            Self::Checkbox(input_type) => input_type as &dyn SpecificInputType,
            Self::Color(input_type) => input_type as &dyn SpecificInputType,
            Self::Date(input_type) => input_type as &dyn SpecificInputType,
            Self::DatetimeLocal(input_type) => input_type as &dyn SpecificInputType,
            Self::Email(input_type) => input_type as &dyn SpecificInputType,
            Self::File(input_type) => input_type as &dyn SpecificInputType,
            Self::Hidden(input_type) => input_type as &dyn SpecificInputType,
            Self::Image(input_type) => input_type as &dyn SpecificInputType,
            Self::Month(input_type) => input_type as &dyn SpecificInputType,
            Self::Number(input_type) => input_type as &dyn SpecificInputType,
            Self::Password(input_type) => input_type as &dyn SpecificInputType,
            Self::Radio(input_type) => input_type as &dyn SpecificInputType,
            Self::Range(input_type) => input_type as &dyn SpecificInputType,
            Self::Reset(input_type) => input_type as &dyn SpecificInputType,
            Self::Search(input_type) => input_type as &dyn SpecificInputType,
            Self::Submit(input_type) => input_type as &dyn SpecificInputType,
            Self::Tel(input_type) => input_type as &dyn SpecificInputType,
            Self::Text(input_type) => input_type as &dyn SpecificInputType,
            Self::Time(input_type) => input_type as &dyn SpecificInputType,
            Self::Url(input_type) => input_type as &dyn SpecificInputType,
            Self::Week(input_type) => input_type as &dyn SpecificInputType,
        }
    }

    /// Defines which input type that should perform like a text input,
    /// specifically when it is interacting with JS. Note that Password
    /// is not included here since it is handled slightly differently,
    /// with placeholder characters shown rather than the underlying value.
    pub(crate) fn is_textual(&self) -> bool {
        matches!(
            *self,
            Self::Date(_) |
                Self::DatetimeLocal(_) |
                Self::Email(_) |
                Self::Hidden(_) |
                Self::Month(_) |
                Self::Number(_) |
                Self::Range(_) |
                Self::Search(_) |
                Self::Tel(_) |
                Self::Text(_) |
                Self::Time(_) |
                Self::Url(_) |
                Self::Week(_)
        )
    }

    pub(crate) fn is_textual_or_password(&self) -> bool {
        self.is_textual() || matches!(self, Self::Password(_))
    }

    /// <https://html.spec.whatwg.org/multipage/#has-a-periodic-domain>
    pub(crate) fn has_periodic_domain(&self) -> bool {
        matches!(self, Self::Time(_))
    }

    pub(crate) fn as_str(&self) -> &str {
        match *self {
            InputType::Button(_) => "button",
            InputType::Checkbox(_) => "checkbox",
            InputType::Color(_) => "color",
            InputType::Date(_) => "date",
            InputType::DatetimeLocal(_) => "datetime-local",
            InputType::Email(_) => "email",
            InputType::File(_) => "file",
            InputType::Hidden(_) => "hidden",
            InputType::Image(_) => "image",
            InputType::Month(_) => "month",
            InputType::Number(_) => "number",
            InputType::Password(_) => "password",
            InputType::Radio(_) => "radio",
            InputType::Range(_) => "range",
            InputType::Reset(_) => "reset",
            InputType::Search(_) => "search",
            InputType::Submit(_) => "submit",
            InputType::Tel(_) => "tel",
            InputType::Text(_) => "text",
            InputType::Time(_) => "time",
            InputType::Url(_) => "url",
            InputType::Week(_) => "week",
        }
    }
}

impl TryFrom<InputType> for InputMethodType {
    type Error = &'static str;

    fn try_from(input_type: InputType) -> Result<Self, Self::Error> {
        match input_type {
            InputType::Color(_) => Ok(InputMethodType::Color),
            InputType::Date(_) => Ok(InputMethodType::Date),
            InputType::DatetimeLocal(_) => Ok(InputMethodType::DatetimeLocal),
            InputType::Email(_) => Ok(InputMethodType::Email),
            InputType::Month(_) => Ok(InputMethodType::Month),
            InputType::Number(_) => Ok(InputMethodType::Number),
            InputType::Password(_) => Ok(InputMethodType::Password),
            InputType::Search(_) => Ok(InputMethodType::Search),
            InputType::Tel(_) => Ok(InputMethodType::Tel),
            InputType::Text(_) => Ok(InputMethodType::Text),
            InputType::Time(_) => Ok(InputMethodType::Time),
            InputType::Url(_) => Ok(InputMethodType::Url),
            InputType::Week(_) => Ok(InputMethodType::Week),
            _ => Err("Input does not support IME."),
        }
    }
}

impl TryFrom<&InputType> for InputMethodType {
    type Error = &'static str;

    fn try_from(input_type: &InputType) -> Result<Self, Self::Error> {
        match input_type {
            InputType::Color(_) => Ok(InputMethodType::Color),
            InputType::Date(_) => Ok(InputMethodType::Date),
            InputType::DatetimeLocal(_) => Ok(InputMethodType::DatetimeLocal),
            InputType::Email(_) => Ok(InputMethodType::Email),
            InputType::Month(_) => Ok(InputMethodType::Month),
            InputType::Number(_) => Ok(InputMethodType::Number),
            InputType::Password(_) => Ok(InputMethodType::Password),
            InputType::Search(_) => Ok(InputMethodType::Search),
            InputType::Tel(_) => Ok(InputMethodType::Tel),
            InputType::Text(_) => Ok(InputMethodType::Text),
            InputType::Time(_) => Ok(InputMethodType::Time),
            InputType::Url(_) => Ok(InputMethodType::Url),
            InputType::Week(_) => Ok(InputMethodType::Week),
            _ => Err("Input does not support IME."),
        }
    }
}

impl From<&Atom> for InputType {
    fn from(value: &Atom) -> InputType {
        match value.to_ascii_lowercase() {
            atom!("button") => Self::button(),
            atom!("checkbox") => Self::checkbox(),
            atom!("color") => Self::color(),
            atom!("date") => Self::date(),
            atom!("datetime-local") => Self::datetime_local(),
            atom!("email") => Self::email(),
            atom!("file") => Self::file(),
            atom!("hidden") => Self::hidden(),
            atom!("image") => Self::image(),
            atom!("month") => Self::month(),
            atom!("number") => Self::number(),
            atom!("password") => Self::password(),
            atom!("radio") => Self::radio(),
            atom!("range") => Self::range(),
            atom!("reset") => Self::reset(),
            atom!("search") => Self::search(),
            atom!("submit") => Self::submit(),
            atom!("tel") => Self::tel(),
            atom!("text") => Self::text(),
            atom!("time") => Self::time(),
            atom!("url") => Self::url(),
            atom!("week") => Self::week(),
            _ => Self::text(),
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

    fn get_files(&self) -> Option<DomRoot<FileList>> {
        None
    }

    fn set_files(&self, _filelist: &FileList) {}

    fn update_shadow_tree(&self, _cx: &mut JSContext, _input: &HTMLInputElement) {}

    fn update_placeholder_contents(&self, _cx: &mut JSContext, _input: &HTMLInputElement) {}

    fn attribute_mutated(
        &self,
        _cx: &mut JSContext,
        _input: &HTMLInputElement,
        _attr: &Attr,
        _mutation: AttributeMutation,
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
