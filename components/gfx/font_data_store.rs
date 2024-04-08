/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::{Arc, Weak}};

use once_cell::sync::OnceCell;
use parking_lot::{Condvar, Mutex, RwLock};
use uluru::LRUCache;

use crate::font_identifier::FontDataIdentifier;


static FONT_DATA_STORE: OnceCell<FontDataStore> = OnceCell::new();

#[derive(Default)]
 struct FontDataStoreLockedData {
    /// Font data that is loaded into memory.
    in_memory_font_data: HashMap<FontDataIdentifier, Weak<Vec<u8>>>,
    /// Identifiers of fonts that are currently loading and a Mutex & CondVar to wait for them to finish loading.
    fonts_currently_loading: HashMap<FontDataIdentifier, Arc<(Mutex<bool>, Condvar)>>,
    /// An LRU cache of fonts to avoid having to constantly reload font data while processing system font lists.
    lru_cache: LRUCache<Arc<Vec<u8>>, 24>,
 }

#[derive(Default)]
/// The [`FontDataStore`] is responsible for loading and storing font data a single time
/// for each process. Either the data is already loaded due to web font loading or if
/// the font is a local font it is loaded and cached. Data that is no longer referenced
/// outside the [`FontDataStore`] may be evicted at any time.
pub(crate) struct FontDataStore {
    locked_data: RwLock<FontDataStoreLockedData>,
 }

impl FontDataStore {
    /// Get the global [`FontDataStore`]. There is one of these per-process.
    pub fn get() -> &'static FontDataStore {
        FONT_DATA_STORE.get_or_init(Default::default)
    }

    /// Get or load the font data for the given identifier. This currently only
    /// works for local font data. Web fonts must be inserted into the [`FontDataStore`]
    /// using [`FontDataStore::insert`].
    pub fn get_or_load_data_for_identifier(&self, identifier: &FontDataIdentifier) -> Arc<Vec<u8>> {
        {
            // If the data already exists in the store, return it directly.
            let readable_data = self.locked_data.read();
            if let Some(font_data) = readable_data.in_memory_font_data.get(identifier).and_then(|weak| weak.upgrade()) {
            //if let Some(font_data) = readable_data.in_memory_font_data.get(identifier) {
                return font_data.clone();
            }
        }

        // Reaching here means the data was not yet loaded.  Now we enter the critical
        // section where font loading might start.
        let rw_store = self.locked_data.write();

        // If the font loaded in the meantime, return it directly.
        if let Some(font_data) = rw_store.in_memory_font_data.get(identifier).and_then(|weak| weak.upgrade()) {
        //if let Some(font_data) = rw_store.in_memory_font_data.get(identifier) {
            return font_data.clone();
        }

        // Either the font is currently loading in which case the data store has a Mutex
        // and Condvar for it, or this thread is responsible for loading the font in which
        // case insert a new Mutex and Condvar and start the loading process.
        if let Some(mutex_and_condvar) = rw_store.fonts_currently_loading.get(identifier).cloned() {
            // This is necessary to unlock the data store.
            drop(rw_store);

            // The font is loading, so wait on the Mutex and the Condvar.
            let (mutex, condvar) = &*mutex_and_condvar.clone();
            let mut loaded = mutex.lock();
            while !*loaded {
                condvar.wait(&mut loaded);
            }

            // The font data should be loaded now, so the next call to this function will return it.
            self.get_or_load_data_for_identifier(identifier)
        } else {
            // The font isn't loading yet, so add the Mutex and Convar for the load and
            // start loading.
            let mutex_and_condvar = Arc::new((Mutex::new(false), Condvar::new()));

            // This is necessary to unlock the data store.
            drop(rw_store);

            let data = self.load_data_for_local_identifier(identifier);

            // Notify anyone else waiting on this font data that it is now loaded.
            let (mutex, condvar) = &*mutex_and_condvar;
            let mut loaded = mutex.lock();
            *loaded = true;
            condvar.notify_all();

            data
        }
    }

    fn load_data_for_local_identifier(&self, identifier: &FontDataIdentifier) -> Arc<Vec<u8>> {
        let FontDataIdentifier::Path(path) = identifier else {
            unreachable!("Unexpected web font without data.")
        };
        println!("loading: {:?}", path);

        let mut bytes = Vec::new();
        File::open(Path::new(&*path.clone()))
            .expect("Couldn't open font file!")
            .read_to_end(&mut bytes)
            .unwrap();
        let font_data = Arc::new(bytes);

        {
            let mut rw_store = self.locked_data.write();
            rw_store.in_memory_font_data.insert(identifier.clone(), Arc::downgrade(&font_data));
            rw_store.lru_cache.insert(font_data.clone());
        }

        font_data
    }
}