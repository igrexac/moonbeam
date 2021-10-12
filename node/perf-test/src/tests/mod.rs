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

use std::time::Duration;

use crate::command::{FullBackend, FullClient, TestContext};

use sc_service::NativeExecutionDispatch;
use service::{Block, RuntimeApiCollection};
use sp_api::ConstructRuntimeApi;

use cli_table::{format::Justify, print_stdout, Cell, Style, Table};

mod block_creation;
mod fibonacci;
mod storage;
pub use block_creation::BlockCreationPerfTest;
pub use fibonacci::FibonacciPerfTest;
pub use storage::StoragePerfTest;

/// struct representing the test results of a single test
#[derive(Default, Clone, Table)]
pub struct TestResults {
	#[table(title = "Test Name")]
	pub test_name: String,
	#[table(
		title = "Overall Time",
		display_fn = "display_duration",
		justify = "Justify::Right"
	)]
	pub overall_duration: Duration,
	#[table(
		title = "Reference",
		display_fn = "display_duration",
		justify = "Justify::Right"
	)]
	pub reference_duration: Duration,
	#[table(title = "Relative", display_fn = "display_relative", justify = "Justify::Right")]
	pub relative: f64,
}

impl TestResults {
	pub fn new(name: &str, duration: Duration, reference_duration: Duration) -> Self {
		let this_run = duration.as_micros() as f64;
		let ref_run = reference_duration.as_micros() as f64;
		let relative = ref_run / this_run;

		TestResults {
			test_name: name.into(),
			overall_duration: duration,
			reference_duration,
			relative,
		}
	}
}

pub trait TestRunner<RuntimeApi, Executor>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	fn run(
		&mut self,
		context: &TestContext<RuntimeApi, Executor>,
	) -> Result<Vec<TestResults>, String>;
}

fn display_duration(duration: &Duration) -> impl std::fmt::Display {
	let ms = duration.as_millis();
	let us = duration.as_micros() % 1000;
	let as_decimal: f64 = ms as f64 + (us as f64 / 1000.0);
	format!("{:.3} ms", as_decimal)
}

fn display_relative(relative: &f64) -> impl std::fmt::Display {
	format!("{:.1} %", relative * 100f64)
}
