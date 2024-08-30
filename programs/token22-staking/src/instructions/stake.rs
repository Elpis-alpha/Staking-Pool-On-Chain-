use {
    crate::{errors::*, state::*, utils::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{transfer_checked, TransferChecked},
        token_interface,
    },
};

pub fn handler_s(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
    check_token_program(ctx.accounts.token_program.key());

    msg!("Pool initial total: {}", ctx.accounts.pool_state.amount);
    msg!(
        "User entry initial balance: {}",
        ctx.accounts.user_stake_entry.balance
    );

    let decimals = ctx.accounts.token_mint.decimals;
    // transfer_checked for either spl-token or the Token Extension program
    transfer_checked(ctx.accounts.transfer_checked_ctx(), stake_amount, decimals)?;

    let pool_state = &mut ctx.accounts.pool_state;
    let user_entry = &mut ctx.accounts.user_stake_entry;

    // update pool state amount
    pool_state.amount = pool_state
        .amount
        .checked_add(stake_amount)
        .expect("Pool overflow");
    msg!("Current pool stake total: {}", pool_state.amount);

    // update user entry balance
    user_entry.balance = user_entry
        .balance
        .checked_add(stake_amount)
        .expect("User entry overflow");
    msg!("User stake balance: {}", user_entry.balance);

    // update user last staked timestamp
    user_entry.last_staked = Clock::get()
        .expect("Clock account not available")
        .unix_timestamp;
    msg!("User last staked: {}", user_entry.last_staked);

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    // pool state
    #[account(
        mut,
        seeds = [token_mint.key().as_ref(), STAKE_POOL_STATE_SEED.as_bytes()],
        bump = pool_state.bump
    )]
    pub pool_state: Account<'info, PoolState>,

    // token mint
    #[account(
        mut,
        mint::token_program = token_program
    )]
    pub token_mint: InterfaceAccount<'info, token_interface::Mint>,

    // pool authority
    /// CHECK: PDA, auth over all token vaults
    #[account(
        seeds = [VAULT_AUTH_SEED.as_bytes()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    // token vault
    #[account(
        mut,
        seeds = [token_mint.key().as_ref(), pool_authority.key().as_ref(), VAULT_SEED.as_bytes()],
        bump = pool_state.vault_bump,
        token::token_program = token_program
    )]
    pub token_vault: InterfaceAccount<'info, token_interface::TokenAccount>,

    // user account
    #[account(
        mut,
        constraint = user.key() == user_stake_entry.user
        @ StakeError::InvalidUser
    )]
    pub user: Signer<'info>,

    // user token account
    #[account(
        mut,
        constraint = user_token_account.mint == pool_state.token_mint
        @ StakeError::InvalidMint,
        token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, token_interface::TokenAccount>,

    // user stake entry
    #[account(
        mut,
        seeds = [user.key().as_ref(), pool_state.token_mint.key().as_ref(), STAKE_ENTRY_SEED.as_bytes()],
        bump = user_stake_entry.bump,
    )]
    pub user_stake_entry: Account<'info, StakeEntry>,

    // other accounts
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
}

impl<'info> Stake<'info> {
    // transfer_checked for spl-token or Token2022
    pub fn transfer_checked_ctx(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.user_token_account.to_account_info(),
            to: self.token_vault.to_account_info(),
            authority: self.user.to_account_info(),
            mint: self.token_mint.to_account_info(),
        };

        return CpiContext::new(cpi_program, cpi_accounts);
    }
}
