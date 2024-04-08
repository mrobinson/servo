/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use style::Atom;

use crate::platform::font_identifier::LocalFontIdentifier;

 /// A unique identifier for a font, which may be a local font or web font. If this is a
 /// local font, this uniquely identifies a particular font file and variation.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FontIdentifier {
    Local(LocalFontIdentifier),
    Web(ServoUrl),
}

/// A unique identifier for font data. This differs from
/// [`crate::font_cache_thread::FontIdentifier`] because two unique identifiers can share
/// the same path if that file contains more than a single variation.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum FontDataIdentifier {
    Url(Atom),
    Path(Atom),
}

impl From<FontIdentifier> for FontDataIdentifier {
    fn from(identifier: FontIdentifier) -> Self {
        match identifier {
            FontIdentifier::Local(ref local) => local.into(),
            FontIdentifier::Web(url) => FontDataIdentifier::Url(url.to_string().into()),
        }
    }
}
