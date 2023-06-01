// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Aleo SDK library.

// The Aleo SDK library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo SDK library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo SDK library. If not, see <https://www.gnu.org/licenses/>.

pub mod deploy;

pub use deploy::*;
use rand::{rngs::StdRng, SeedableRng};
use std::str::FromStr;

pub mod execute;
pub use execute::*;

pub mod join;
pub use join::*;

pub mod split;
pub use split::*;

pub mod transfer;
pub use transfer::*;

pub mod utils;

use crate::{
    types::{
        CurrentAleo,
        IdentifierNative,
        ProcessNative,
        ProgramIDNative,
        ProgramNative,
        ProvingKeyNative,
        VerifyingKeyNative,
    },
    ProvingKey,
    RecordPlaintext,
    VerifyingKey,
};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ProgramManager {
    process: ProcessNative,
}

#[wasm_bindgen]
impl ProgramManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { process: ProcessNative::load_web().unwrap() }
    }

    /// Validate that an amount being paid from a record is greater than zero and that the record
    /// has enough credits to pay the amount
    pub(crate) fn validate_amount(credits: f64, amount: &RecordPlaintext, fee: bool) -> Result<u64, String> {
        let name = if fee { "Fee" } else { "Amount" };

        if credits <= 0.0 {
            return Err(format!("{name} must be greater than zero to deploy or execute a program"));
        }
        let microcredits = (credits * 1_000_000.0f64) as u64;
        if amount.microcredits() < microcredits {
            return Err(format!("{name} record does not have enough credits to pay the specified fee"));
        }

        Ok(microcredits)
    }

    /// Cache the proving and verifying keys for a program function in WASM memory. This method
    /// will take a verifying and proving key and store them in the program manager's internal
    /// in-memory cache. This memory is allocated in WebAssembly, so it is important to be mindful
    /// of the amount of memory being used.
    ///
    /// @param program_id The name of the program containing the desired function
    /// @param function The name of the function to store the keys for
    /// @param proving_key The proving key of the function
    /// @param verifying_key The verifying key of the function
    #[wasm_bindgen(js_name = "cacheKeysInWasmMemory")]
    pub fn cache_keys_in_wasm_memory(
        &mut self,
        program_id: &str,
        function: &str,
        proving_key: ProvingKey,
        verifying_key: VerifyingKey,
    ) -> Result<(), String> {
        let program_id = ProgramIDNative::from_str(program_id).map_err(|e| e.to_string())?;
        let function = IdentifierNative::from_str(function).map_err(|e| e.to_string())?;

        self.process
            .insert_proving_key(&program_id, &function, ProvingKeyNative::from(proving_key))
            .map_err(|e| e.to_string())?;

        self.process
            .insert_verifying_key(&program_id, &function, VerifyingKeyNative::from(verifying_key))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Get the proving & verifying keys cached in WASM memory for a specific function
    ///
    /// @param program_id The name of the program containing the desired function
    /// @param function_id The name of the function to retrieve the keys for
    #[wasm_bindgen(js_name = "getCachedKey")]
    pub fn get_cached_keypair(&self, program_id: &str, function: &str) -> Result<js_sys::Array, String> {
        let program_id = ProgramIDNative::from_str(program_id).map_err(|e| e.to_string())?;
        let function_id = IdentifierNative::from_str(function).map_err(|e| e.to_string())?;
        self.get_keypair(&self.process, &program_id, &function_id)
    }

    /// Get a keypair from a process
    pub(crate) fn get_keypair(
        &self,
        process: &ProcessNative,
        program_id: &ProgramIDNative,
        function_id: &IdentifierNative,
    ) -> Result<js_sys::Array, String> {
        let proving_key =
            ProvingKey::from(process.get_proving_key(program_id, function_id).map_err(|e| e.to_string())?);
        let verifying_key =
            VerifyingKey::from(process.get_verifying_key(program_id, function_id).map_err(|e| e.to_string())?);
        let array = js_sys::Array::new();
        array.push(&proving_key.into());
        array.push(&verifying_key.into());
        Ok(array)
    }

    /// Synthesize a proving and verifying key for a program function. This method should be used
    /// when there is a need to pre-synthesize keys (i.e. for caching purposes, etc.)
    ///
    /// @param program The source code of the program containing the desired function
    /// @param function The name of the function to synthesize the key for
    #[wasm_bindgen(js_name = "synthesizeKeypair")]
    pub fn synthesize_keypair(&mut self, program: &str, function: &str) -> Result<js_sys::Array, String> {
        let mut process = ProcessNative::load_web().map_err(|e| e.to_string())?;
        let program = ProgramNative::from_str(program).map_err(|e| e.to_string())?;
        let function_id = IdentifierNative::from_str(function).map_err(|e| e.to_string())?;
        let program_id = program.id();
        if &program_id.to_string() != "credits.aleo" {
            process.add_program(&program).map_err(|e| e.to_string())?;
        }

        process
            .synthesize_key::<CurrentAleo, _>(program_id, &function_id, &mut StdRng::from_entropy())
            .map_err(|e| e.to_string())?;
        self.get_keypair(&process, program_id, &function_id)
    }
}

impl Default for ProgramManager {
    fn default() -> Self {
        Self::new()
    }
}
