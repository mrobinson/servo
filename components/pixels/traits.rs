/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{CorsStatus, RasterImage};

#[typetag::serde(tag = "type")]
pub trait ServoImageDecoder: Sync + Send {
    fn decode_image(&self, buffer: &[u8], cors_status: CorsStatus) -> Option<RasterImage>;
}
