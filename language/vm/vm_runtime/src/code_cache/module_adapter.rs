// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Fetches code data from the blockchain.

use logger::prelude::*;
use state_view::StateView;
use std::collections::HashMap;
use types::language_storage::CodeKey;
use vm::file_format::CompiledModule;

/// Trait that describes how the VM expects code data to be stored.
pub trait ModuleFetcher {
    /// `CodeKey` is the fully qualified name for the module we are trying to fetch.
    fn get_module(&self, key: &CodeKey) -> Option<CompiledModule>;
}

/// A wrapper around State Store database for fetching code data stored on chain.
pub struct ModuleFetcherImpl<'a>(&'a dyn StateView);

impl<'a> ModuleFetcherImpl<'a> {
    /// Creates a new Fetcher instance with a `StateView` reference.
    pub fn new(storage: &'a dyn StateView) -> Self {
        ModuleFetcherImpl(storage)
    }
}

impl<'a> ModuleFetcher for ModuleFetcherImpl<'a> {
    fn get_module(&self, key: &CodeKey) -> Option<CompiledModule> {
        let access_path = key.into();
        match self.0.get(&access_path) {
            Ok(opt_module_blob) => match opt_module_blob {
                Some(module_blob) => match CompiledModule::deserialize(&module_blob) {
                    Ok(module) => Some(module),
                    Err(_) => {
                        crit!(
                            "[VM] Storage contains a malformed module with key {:?}",
                            key
                        );
                        None
                    }
                },
                None => {
                    crit!("[VM] Storage returned None for module with key {:?}", key);
                    None
                }
            },
            Err(_) => {
                crit!("[VM] Error fetching module with key {:?}", key);
                None
            }
        }
    }
}

/// A wrapper for an empty state with no code data stored.
pub struct NullFetcher();

impl ModuleFetcher for NullFetcher {
    fn get_module(&self, _key: &CodeKey) -> Option<CompiledModule> {
        None
    }
}

/// A wrapper for a state with a list of pre-compiled modules.
pub struct FakeFetcher(HashMap<CodeKey, CompiledModule>);

impl FakeFetcher {
    /// Create a FakeFetcher instance with a vector of pre-compiled modules.
    pub fn new(modules: Vec<CompiledModule>) -> Self {
        let mut map = HashMap::new();
        for m in modules.into_iter() {
            map.insert(m.self_code_key(), m);
        }
        FakeFetcher(map)
    }

    /// Remove all modules stored in the fetcher.
    pub fn clear(&mut self) {
        self.0 = HashMap::new();
    }
}

impl ModuleFetcher for FakeFetcher {
    fn get_module(&self, key: &CodeKey) -> Option<CompiledModule> {
        self.0.get(key).cloned()
    }
}