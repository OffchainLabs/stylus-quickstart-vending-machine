//!
//! Stylus Cupcake Example
//!
//! The contract is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!

// Allow `cargo stylus export-abi` to generate a main function if the "export-abi" feature is enabled.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::alloy_primitives::{Address, U256};
use stylus_sdk::console;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct VendingMachine {
        mapping(address => uint256) cupcake_balances;
        mapping(address => uint256) cupcake_distribution_times;
    }
}

#[public]
impl VendingMachine {
    pub fn give_cupcake_to(&mut self, user_address: Address) -> Result<bool, Vec<u8>> {
        // Get the last distribution time for the user.
        let last_distribution = self.cupcake_distribution_times.get(user_address);
        // Calculate the earliest next time the user can receive a cupcake.
        let five_seconds_from_last_distribution = last_distribution + U256::from(5);

        // Get the current block timestamp using the VM pattern
        let current_time = self.vm().block_timestamp();
        // Check if the user can receive a cupcake.
        let user_can_receive_cupcake =
            five_seconds_from_last_distribution <= U256::from(current_time);

        if user_can_receive_cupcake {
            // Increment the user's cupcake balance.
            let mut balance_accessor = self.cupcake_balances.setter(user_address);
            let balance = balance_accessor.get() + U256::from(1);
            balance_accessor.set(balance);

            // Get current timestamp using the VM pattern BEFORE creating the mutable borrow
            let new_distribution_time = self.vm().block_timestamp();

            // Update the distribution time to the current time.
            let mut time_accessor = self.cupcake_distribution_times.setter(user_address);
            time_accessor.set(U256::from(new_distribution_time));
            return Ok(true);
        } else {
            // User must wait before receiving another cupcake.
            console!(
                "HTTP 429: Too Many Cupcakes (you must wait at least 5 seconds between cupcakes)"
            );
            return Ok(false);
        }
    }
    pub fn get_cupcake_balance_for(&self, user_address: Address) -> Result<U256, Vec<u8>> {
        Ok(self.cupcake_balances.get(user_address))
    }
}
