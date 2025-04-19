//!
//! Stylus Cupcake Example
//!
//! The contract is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!

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

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::address;
    use stylus_sdk::testing::*;

    #[test]
    fn test_give_cupcake_to() {
        let vm: TestVM = TestVMBuilder::new()
            .sender(address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9"))
            .contract_address(address!("0x11b57fe348584f042e436c6bf7c3c3def171de49"))
            .value(U256::from(1))
            .build();
        let mut contract = VendingMachine::from(&vm);
        let user = address!("0xCDC41bff86a62716f050622325CC17a317f99404");
        assert_eq!(contract.get_cupcake_balance_for(user).unwrap(), U256::ZERO);

        vm.set_block_timestamp(vm.block_timestamp() + 6);

        // Give a cupcake and verify it succeeds
        assert!(contract.give_cupcake_to(user).unwrap());

        // Check balance is now 1
        assert_eq!(
            contract.get_cupcake_balance_for(user).unwrap(),
            U256::from(1)
        );

        // Try to give another cupcake immediately - should fail due to time restriction
        assert!(!contract.give_cupcake_to(user).unwrap());

        // Balance should still be 1
        assert_eq!(
            contract.get_cupcake_balance_for(user).unwrap(),
            U256::from(1)
        );

        // Advance block timestamp by 6 seconds
        vm.set_block_timestamp(vm.block_timestamp() + 6);

        // Now giving a cupcake should succeed
        assert!(contract.give_cupcake_to(user).unwrap());

        // Balance should now be 2
        assert_eq!(
            contract.get_cupcake_balance_for(user).unwrap(),
            U256::from(2)
        );
    }
}
