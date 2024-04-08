/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::io::Error;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use webrender_api::NativeFontHandle;

use crate::font_data_store::FontDataStore;
use crate::font_identifier::FontIdentifier;

/// Platform specific font representation for Linux.
#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    /// Lazily-loaded (for local fonts) byte data that can be passed
    /// to Freetype or Raqote directly.
    pub font_data: RwLock<Option<Arc<Vec<u8>>>>,
    pub identifier: FontIdentifier,
}

impl fmt::Debug for FontTemplateData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FontTemplateData")
            .field("identifier", &self.identifier)
            .field(
                "font_data",
                &self
                    .font_data
                    .read()
                    .unwrap()
                    .as_ref()
                    .map(|bytes| format!("[{} bytes]", bytes.len())),
            )
            .finish()
    }
}

impl FontTemplateData {
    pub fn new(
        identifier: FontIdentifier,
        font_data: Option<Vec<u8>>,
    ) -> Result<FontTemplateData, Error> {
        Ok(FontTemplateData {
            identifier,
            font_data: RwLock::new(font_data.map(Arc::new)),
        })
    }

    /// Returns a reference to the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Arc<Vec<u8>> {
        self.font_data
            .write()
            .unwrap()
            .get_or_insert_with(|| {
                FontDataStore::get().get_or_load_data_for_identifier(&self.identifier.clone().into())
            })
            .clone()
    }

    /// Returns a reference to the bytes in this font if they are in memory. This function
    /// never performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Arc<Vec<u8>>> {
        self.font_data.read().unwrap().as_ref().cloned()
    }

    /// Returns the native font that underlies this font template, if applicable.
    pub fn native_font(&self) -> Option<NativeFontHandle> {
        let local_identifier = match &self.identifier {
            FontIdentifier::Local(local_identifier) => local_identifier,
            FontIdentifier::Web(_) => return None,
        };

        Some(NativeFontHandle {
            path: PathBuf::from(&*local_identifier.path),
            index: 0,
        })
    }
}
