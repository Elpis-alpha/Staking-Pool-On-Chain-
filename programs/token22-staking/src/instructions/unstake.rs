use {
    crate::{errors::*, state::*, utils::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{mint_to, transfer_checked, MintTo, TransferChecked},
        token_interface,
    },
};

pub fn handler_u(ctx: Context<Unstake>) -> Result<()> {
    check_token_program(ctx.accounts.token_program.key());

    let user_entry = &ctx.accounts.user_stake_entry;
    let amount = user_entry.balance;
    let decimals = ctx.accounts.token_mint.decimals;
    let total_pool_balance = ctx.accounts.pool_state.amount;
    let user_balance_before_withdrawal = ctx.accounts.user_token_account.amount;

    msg!("User staked amount: {}", amount);
    msg!(
        "User tokens before withdrawal: {}",
        user_balance_before_withdrawal
    );
    msg!("Total staked before withdrawal: {}", total_pool_balance);

    if amount > total_pool_balance {
        return Err(StakeError::OverdrawError.into());
    }

    let auth_bump = ctx.accounts.pool_state.vault_auth_bump;
    let auth_seeds = &[VAULT_AUTH_SEED.as_bytes(), &[auth_bump]];
    let signer = &[&auth_seeds[..]];

    // transfer staked tokens
    transfer_checked(ctx.accounts.transfer_checked_ctx(signer), amount, decimals)?;

    // mint users staking rewards, 10x amount of staked tokens
    let stake_rewards = amount.checked_mul(10).expect("Stake rewards mul overflow");

    // mint rewards to user
    mint_to(ctx.accounts.mint_to_ctx(signer), stake_rewards)?;

    // get refs
    let pool_state = &mut ctx.accounts.pool_state;
    let user_entry = &mut ctx.accounts.user_stake_entry;

    // update pool state amount
    pool_state.amount = pool_state
        .amount
        .checked_sub(amount)
        .expect("Pool amount sub underflow");
    msg!("Total staked after withdrawal: {}", pool_state.amount);

    // update user entry balance
    user_entry.balance = user_entry
        .balance
        .checked_sub(amount)
        .expect("User balance sub underflow");
    msg!(
        "User staked balance after withdrawal: {}",
        user_entry.balance
    );

    // update user last staked timestamp
    user_entry.last_staked = Clock::get()
        .expect("Clock account not available")
        .unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
pub struct Unstake<'info> {
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

    // starting token mint
    #[account(
        mut,
        mint::authority = pool_authority,
        mint::token_program = token_program,
        constraint = staking_token_mint.key() == pool_state.staking_token_mint
        @ StakeError::InvalidStakingTokenMint
    )]
    pub staking_token_mint: InterfaceAccount<'info, token_interface::Mint>,

    // user stake token account
    #[account(
        mut,
        token::authority = user,
        token::mint = staking_token_mint,
        token::token_program = token_program,
        constraint = user_stake_token_account.key() == user_stake_entry.user_stake_token_account
        @ StakeError::InvalidUserStakeTokenAccount
    )]
    pub user_stake_token_account: InterfaceAccount<'info, token_interface::TokenAccount>,

    // other accounts
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
}

impl<'info> Unstake<'info> {
    // transfer_checked for Token2022
    pub fn transfer_checked_ctx<'a>(
        &'a self,
        seeds: &'a [&[&[u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.token_vault.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.pool_authority.to_account_info(),
            mint: self.token_mint.to_account_info(),
        };

        return CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    }

    // mint to
    pub fn mint_to_ctx<'a>(
        &'a self,
        seeds: &'a [&[&[u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.staking_token_mint.to_account_info(),
            to: self.user_stake_token_account.to_account_info(),
            authority: self.pool_authority.to_account_info(),
        };

        return CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    }
}
