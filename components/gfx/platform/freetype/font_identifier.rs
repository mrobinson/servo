/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */


use serde::{Deserialize, Serialize};
use style::Atom;

use crate::font_identifier::FontDataIdentifier;


/// An identifier for a local font on systems using Freetype.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
    /// The variation index within the font.
    pub variation_index: i32,
}


impl From<&LocalFontIdentifier> for FontDataIdentifier {
    fn from(identifier: &LocalFontIdentifier) -> Self {
        FontDataIdentifier::Path(identifier.path.clone())
    }
}