// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Environmental-aware externalities for EVM tracing in Wasm runtime. This enables
//! capturing the - potentially large - trace output data in the host and keep
//! a low memory footprint in `--execution=wasm`.
//!
//! - The original trace Runtime Api call is wrapped `using` environmental (thread local).
//! - Arguments are scale-encoded known types in the host.
//! - Host functions will decode the input and emit an event `with` environmental.

#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime_interface::runtime_interface;

use codec::Decode;
use sp_std::vec::Vec;

use moonbeam_rpc_primitives_debug::{
	events::{Event as EventV2, EvmEvent, GasometerEvent, RuntimeEvent},
};

#[runtime_interface]
pub trait MoonbeamExt {
	// New design, proxy events.
	/// An `Evm` event proxied by the Moonbeam runtime to this host function.
	/// evm -> moonbeam_runtime -> host.
	fn evm_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = EvmEvent::decode(&mut &event[..]) {
			EventV2::Evm(event).emit();
		} else {
			tracing::warn!("Failed to decode EvmEvent from bytes : {:?}", event);
		}
	}
	/// A `Gasometer` event proxied by the Moonbeam runtime to this host function.
	/// evm_gasometer -> moonbeam_runtime -> host.
	fn gasometer_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = GasometerEvent::decode(&mut &event[..]) {
			EventV2::Gasometer(event).emit();
		} else {
			tracing::warn!("Failed to decode GasometerEvent from bytes : {:?}", event);
		}
	}
	/// A `Runtime` event proxied by the Moonbeam runtime to this host function.
	/// evm_runtime -> moonbeam_runtime -> host.
	fn runtime_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = RuntimeEvent::decode(&mut &event[..]) {
			EventV2::Runtime(event).emit();
		} else {
			tracing::warn!("Failed to decode RuntimeEvent from bytes : {:?}", event);
		}
	}
	/// An event to create a new CallList (currently a new transaction when tracing a block).
	fn call_list_new(&mut self) {
		EventV2::CallListNew().emit();
	}
}
