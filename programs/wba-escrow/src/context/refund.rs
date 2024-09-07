use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    maker: Signer<'info>,
    #[account(
        mut,
        associated_token::authority = maker,
        associated_token::mint = escrow.mint_x
    )]
    maker_ata_x: InterfaceAccount<'info, TokenAccount>,
    mint_x: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump,
        close = maker,
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::authority = escrow,
        associated_token::mint = escrow.mint_x,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    token_program: Interface<'info, TokenInterface>,
}

impl<'info> Refund<'info> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_x.to_account_info(),
            mint: self.mint_x.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();

        let maker_key = self.maker.to_account_info().key();
        let seed_bytes = self.escrow.seed.to_le_bytes();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            &[self.escrow.escrow_bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_context, self.escrow.amount_x, self.mint_x.decimals)
    }

    pub fn close_vault(&mut self) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        let maker_key = self.maker.to_account_info().key();
        let seed_bytes = self.escrow.seed.to_le_bytes();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            &[self.escrow.escrow_bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        close_account(cpi_context)
    }
}
