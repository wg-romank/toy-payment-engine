use std::collections::HashMap;

use crate::{types::{TransactionId, Currency}, engine::Transaction};


pub struct Account {
    deposits: HashMap<TransactionId, (Currency, bool)>,
    pub total: Currency,
    pub held: Currency,
    pub locked: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            deposits: HashMap::new(),
            total: 0.0,
            held: 0.0,
            locked: false,
        }
    }
}

impl Account {
    pub fn availible(&self) -> Currency {
        self.total - self.held
    }

    pub fn process(&mut self, transaction: Transaction) {
        use Transaction::*;
        if !self.locked {
            match transaction {
                Deposit { id, amount } => {
                    self.deposits.insert(id, (amount, false));
                    self.total += amount;
                }
                // dispute withdrawal?
                Withdrawal { id: _, amount } if amount <= self.availible() => {
                    self.total -= amount;
                }
                Dispute { id } =>
                // to do > available ?
                {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if !*disputed {
                            self.held += *amount;
                            *disputed = true;
                        }
                    }
                }
                Resolve { id } => {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if *disputed {
                            self.held -= *amount;
                            *disputed = false;
                        }
                    }
                }
                Chargeback { id } => {
                    if let Some((amount, disputed)) = self.deposits.get_mut(&id) {
                        if *disputed {
                            self.total -= *amount;
                            self.held -= *amount;
                            *disputed = false;
                            self.locked = true;
                        }
                    }
                }
                t => println!("invalid transaction {:?}", t),
            }
        }
    }
}
