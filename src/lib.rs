#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", feature(allocator_api, global_asm))]

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke,
};
use spl_token::instruction::transfer; // Corrected import

// Entry point for the smart contract
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Token accounts for the standard token (SPL Token)
    let from_token_account = next_account_info(accounts_iter)?;
    let to_token_account = next_account_info(accounts_iter)?;

    // Token accounts for the NFTs
    let from_nft_account = next_account_info(accounts_iter)?;
    let to_nft_account = next_account_info(accounts_iter)?;

    // Deployer wallet (W1) to handle tax
    let deployer_wallet = next_account_info(accounts_iter)?;

    // Tax parameters (adjust these based on your requirements)
    let tax_percent = 5; // 5% tax

    // Ensure the tax percent is valid
    if tax_percent > 100 {
        return Err(ProgramError::InvalidArgument);
    }

    // Calculate tax amount
    let tax_amount = (from_token_account.amount * tax_percent) / 100;

    // Ensure the tax amount doesn't exceed the token balance
    let taxed_amount = std::cmp::min(tax_amount, from_token_account.amount);

    // Transfer custom token from from_token_account to to_token_account with tax
    let token_transfer_ix = transfer(
        &spl_token::id(), // Corrected the program id reference
        from_token_account.key,
        to_token_account.key,
        program_id, // Corrected the program id reference
        &[from_token_account.key],
        from_token_account.amount - taxed_amount, // Transfer net amount after tax
    )?;
    invoke(
        &token_transfer_ix,
        &[from_token_account.clone(), to_token_account.clone()], // Corrected the account references
        &[&from_token_account.key],
    )?;

    // Transfer NFT from from_nft_account to to_nft_account
    let nft_transfer_ix = transfer(
        &spl_token::id(), // Corrected the program id reference
        from_nft_account.key,
        to_nft_account.key,
        program_id, // Corrected the program id reference
        &[from_nft_account.key],
        1, // Transfer one NFT
    )?;
    invoke(
        &nft_transfer_ix,
        &[from_nft_account.clone(), to_nft_account.clone()], // Corrected the account references
        &[&from_nft_account.key],
    )?;

    // Handle tax by transferring taxed_amount to the deployer_wallet
    if taxed_amount > 0 {
        let tax_transfer_ix = transfer(
            &spl_token::id(), // Corrected the program id reference
            from_token_account.key,
            deployer_wallet.key,
            program_id, // Corrected the program id reference
            &[from_token_account.key],
            taxed_amount,
        )?;
        invoke(
            &tax_transfer_ix,
            &[from_token_account.clone(), deployer_wallet.clone()], // Corrected the account references
            &[&from_token_account.key],
        )?;
    }

    Ok(())
}
