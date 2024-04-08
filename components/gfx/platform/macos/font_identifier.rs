/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */


use serde::{Deserialize, Serialize};
use style::Atom;

use crate::font_identifier::FontDataIdentifier;


/// An identifier for a local font on a MacOS system. These values comes from the CoreText
/// CTFontCollection. Note that `path` here is required. We do not load fonts that do not
/// have paths.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    pub postscript_name: Atom,
    pub path: Atom,
}


impl From<&LocalFontIdentifier> for FontDataIdentifier {
    fn from(identifier: &LocalFontIdentifier) -> Self {
        FontDataIdentifier::Path(identifier.path.clone())
    }
}