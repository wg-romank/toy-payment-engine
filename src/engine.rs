use std::collections::HashMap;

use crate::{
    account::Account,
    types::{AccountId, Currency, TransactionId},
};

#[derive(Debug)]
pub enum Transaction {
    Deposit { id: TransactionId, amount: Currency },
    Withdrawal { id: TransactionId, amount: Currency },
    Dispute { id: TransactionId },
    Resolve { id: TransactionId },
    Chargeback { id: TransactionId },
}

pub fn deposit(id: TransactionId, amount: Currency) -> Transaction {
    Transaction::Deposit { id, amount }
}

pub fn withdrawal(id: TransactionId, amount: Currency) -> Transaction {
    Transaction::Withdrawal { id, amount }
}

pub fn dispute(id: TransactionId) -> Transaction {
    Transaction::Dispute { id }
}

pub fn resolve(id: TransactionId) -> Transaction {
    Transaction::Resolve { id }
}

pub fn chargeback(id: TransactionId) -> Transaction {
    Transaction::Chargeback { id }
}

pub struct Engine {
    accounts: HashMap<AccountId, Account>,
}

impl Engine {
    pub fn empty() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, aid: AccountId, t: Transaction) -> Result<(), String> {
        self.accounts
            .entry(aid)
            .or_default()
            .update(t)
            .map_err(|e| format!("account ({}): {}", aid, e))
    }

    pub fn accounts(&self) -> impl Iterator<Item = (&AccountId, &Account)> {
        self.accounts.iter()
    }
}

#[cfg(test)]
mod engine_test {
    use rust_decimal_macros::dec;

    use crate::{account::Account, types::Currency};

    use super::{Engine, Transaction};

    const DEPOSIT_AMOUNT: Currency = dec!(10.);

    fn assert_deposit(f: impl Fn(&Account) -> ()) {
        let mut e = Engine::empty();
        let aid = 1;
        let t = Transaction::Deposit {
            id: 1,
            amount: DEPOSIT_AMOUNT,
        };
        e.process(aid, t)
            .expect("something is wrong, cannot deposit");

        if let Some(acc) = e.accounts.get(&aid) {
            f(acc);
        } else {
            panic!("account {} does not exists", aid);
        }
    }

    #[test]
    fn unmentioned_accounts_get_created() {
        assert_deposit(|acc| assert_eq!(acc.total, DEPOSIT_AMOUNT));
    }

    #[test]
    fn new_accounts_are_not_locked() {
        assert_deposit(|acc| assert_eq!(acc.locked, false));
    }

    #[test]
    fn new_accounts_have_all_funds_available() {
        assert_deposit(|acc| {
            assert_eq!(acc.total, acc.availible());
            assert_eq!(acc.held, dec!(0.0));
        });
    }
}
