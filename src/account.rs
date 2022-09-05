use std::collections::HashMap;

use rust_decimal_macros::dec;

use crate::{
    engine::Transaction,
    types::{Currency, TransactionId},
};

#[derive(Default)]
pub struct Account {
    // there should be a limit on how long we keep those transactions
    // since otherwise bundled in a service this will eventually blow memory up
    // but since the limit is unspecified we prefer correctness assuming
    // that disputes can happen any time for any transaction
    deposits: HashMap<TransactionId, (Currency, bool)>,
    pub total: Currency,
    pub held: Currency,
    pub locked: bool,
}

impl Account {
    pub fn availible(&self) -> Currency {
        self.total - self.held
    }

    // todo: allocating a string on error is sub-optimal
    pub fn update(&mut self, transaction: Transaction) -> Result<(), String> {
        use Transaction::*;
        if !self.locked {
            match transaction {
                Deposit { id, amount } if amount >= dec!(0.) => {
                    self.deposits.insert(id, (amount, false));
                    self.total += amount;
                    Ok(())
                }
                // given the description we assume dispute processing only for deposits
                Withdrawal { id: _, amount }
                    if amount >= dec!(0.) && amount <= self.availible() =>
                {
                    self.total -= amount;
                    Ok(())
                }
                Dispute { id } => {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if !*disputed {
                            self.held += *amount;
                            *disputed = true;
                            Ok(())
                        } else {
                            Err(format!("transaction ({}) is already in dispute", id))
                        }
                    } else {
                        Err(format!("transaction ({}) not found", id))
                    }
                }
                Resolve { id } => {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if *disputed {
                            self.held -= *amount;
                            *disputed = false;
                            Ok(())
                        } else {
                            Err(format!("transaction ({}) is not in dispute", id))
                        }
                    } else {
                        Err(format!("transaction ({}) not found", id))
                    }
                }
                Chargeback { id } => {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if *disputed {
                            if *amount <= self.total {
                                self.total -= *amount;
                                self.held -= *amount;
                                *disputed = false;
                                self.locked = true;
                                Ok(())
                            } else {
                                Err(format!(
                                    "not enough funds to chargeback {} < {}",
                                    self.total, *amount
                                ))
                            }
                        } else {
                            Err(format!("transaction ({}) is not in dispute", id))
                        }
                    } else {
                        Err(format!("transaction ({}) not found", id))
                    }
                }
                t => Err(format!("invalid transaction {:?}", t)),
            }
        } else {
            Err("account is locked".to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::Account;
    use crate::engine::*;

    fn account(transactions: Vec<Transaction>) -> Account {
        transactions
            .into_iter()
            .fold(Account::default(), |mut acc, t| {
                acc.update(t).expect("failed to bootstrap account");
                acc
            })
    }

    #[test]
    fn deposist_increases_total_funds() {
        let acc = account(vec![deposit(1, dec!(10))]);

        assert_eq!(acc.total, dec!(10.));
    }

    #[test]
    fn negative_deposits_do_nothing() {
        let mut acc = Account::default();

        assert!(acc.update(deposit(1, dec!(-10.))).is_err());

        assert_eq!(acc.total, dec!(0.));
    }

    #[test]
    fn withdrawal_decreases_total_funds() {
        let acc = account(vec![deposit(1, dec!(10.)), withdrawal(2, dec!(5.))]);

        assert_eq!(acc.total, dec!(5.));
    }

    #[test]
    fn withdrawal_does_nothing_if_not_enough_funds() {
        let mut acc = Account::default();

        assert!(acc.update(withdrawal(1, dec!(10.))).is_err());

        assert_eq!(acc.total, dec!(0.));
    }

    #[test]
    fn negative_withdrawals_do_nothing() {
        let mut acc = account(vec![deposit(1, dec!(10.))]);

        assert!(acc.update(withdrawal(2, -dec!(10.))).is_err());

        assert_eq!(acc.total, dec!(10.));
    }

    #[test]
    fn dispute_increases_held_funds() {
        let acc = account(vec![deposit(1, dec!(10.)), dispute(1)]);

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(10.));
        assert_eq!(acc.availible(), dec!(0.));
    }

    #[test]
    fn dispute_does_nothing_if_id_does_not_match() {
        let mut acc = account(vec![deposit(1, dec!(10.))]);

        assert!(acc.update(dispute(2)).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(0.));
        assert_eq!(acc.availible(), dec!(10.));
    }

    #[test]
    fn resolve_releases_funds() {
        let acc = account(vec![deposit(1, dec!(10.)), dispute(1), resolve(1)]);

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(0.));
        assert_eq!(acc.availible(), dec!(10.));
    }

    #[test]
    fn resolve_does_nothing_if_id_does_not_match() {
        let mut acc = account(vec![deposit(1, dec!(10.)), dispute(1)]);

        assert!(acc.update(resolve(2)).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(10.));
        assert_eq!(acc.availible(), dec!(0.));
    }

    #[test]
    fn resolve_does_nothing_if_not_in_dispute() {
        let mut acc = account(vec![deposit(1, dec!(10.))]);

        assert!(acc.update(resolve(1)).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(0.));
        assert_eq!(acc.availible(), dec!(10.));
    }

    #[test]
    fn chargeback_locks_account() {
        let acc = account(vec![deposit(1, dec!(10.)), dispute(1), chargeback(1)]);

        assert_eq!(acc.total, dec!(0.));
        assert_eq!(acc.held, dec!(0.));
        assert_eq!(acc.availible(), dec!(0.));
        assert_eq!(acc.locked, true);
    }

    #[test]
    fn chargeback_does_nothing_if_ids_do_not_match() {
        let mut acc = account(vec![deposit(1, dec!(10.)), dispute(1)]);

        assert!(acc.update(chargeback(2)).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.held, dec!(10.));
        assert_eq!(acc.availible(), dec!(0.));
        assert_eq!(acc.locked, false);
    }

    fn locked_account() -> Account {
        account(vec![
            deposit(1, dec!(10.)),
            deposit(2, dec!(5.)),
            dispute(2),
            chargeback(2),
        ])
    }

    #[test]
    fn deposit_does_not_work_on_locked_account() {
        let mut acc = locked_account();

        assert!(acc.update(deposit(1, dec!(10.))).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.locked, true);
    }

    #[test]
    fn withdrawal_does_not_work_on_locked_account() {
        let mut acc = locked_account();

        assert!(acc.update(withdrawal(1, dec!(10.))).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.locked, true);
    }

    #[test]
    fn dispute_does_not_work_on_locked_account() {
        let mut acc = locked_account();

        assert!(acc.update(dispute(1)).is_err());

        assert_eq!(acc.total, dec!(10.));
        assert_eq!(acc.locked, true);
    }

    #[test]
    fn disputing_can_lead_to_negative_balance_and_be_resolved() {
        let mut acc = account(vec![
            deposit(1, dec!(10.)),
            withdrawal(2, dec!(8.)),
            dispute(1),
        ]);

        assert!(acc.availible() < dec!(0.));

        assert!(acc.update(resolve(1)).is_ok());

        assert!(acc.availible() >= dec!(0.));
    }

    #[test]
    fn chargeback_should_not_leave_negative_balance() {
        let mut acc = account(vec![
            deposit(1, dec!(10.)),
            withdrawal(2, dec!(8.)),
            dispute(1),
        ]);

        assert!(acc.update(chargeback(1)).is_err());
    }
}
