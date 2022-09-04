use std::{collections::HashMap, error::Error, env::args, io};

use serde::{Deserialize, Serialize};

type TransactionId = u32;
type AccountId = u16;
type Currency = f32;

struct Account {
    deposits: HashMap<TransactionId, (Currency, bool)>,
    total: Currency,
    held: Currency,
    locked: bool,
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
    fn availible(&self) -> Currency {
        self.total - self.held
    }

    fn process(&mut self, transaction: Transaction) {
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

struct Engine {
    accounts: HashMap<AccountId, Account>,
}

impl Engine {
    fn empty() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
    fn process(&mut self, aid: AccountId, t: Transaction) {
        self.accounts.entry(aid).or_default().process(t);
    }
}

#[derive(Debug, Serialize)]
struct AccountStorage {
    client: AccountId,
    available: Currency,
    held: Currency,
    total: Currency,
    locked: bool,
}

impl AccountStorage {
    fn from(account: (&AccountId, &Account)) -> Self {
        let (id, a) = account;
        Self {
            client: *id,
            available: a.availible(),
            held: a.held,
            total: a.total,
            locked: a.locked,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionRaw {
    #[serde(rename = "type")]
    typ: String,
    client: AccountId,
    tx: TransactionId,
    amount: Option<Currency>,
}

#[derive(Debug)]
enum Transaction {
    Deposit { id: TransactionId, amount: Currency },
    Withdrawal { id: TransactionId, amount: Currency },
    Dispute { id: TransactionId },
    Resolve { id: TransactionId },
    Chargeback { id: TransactionId },
}

impl Transaction {
    fn from(record: TransactionRaw) -> Result<(AccountId, Transaction), String> {
        use Transaction::*;

        match (record.typ.as_str(), record.amount) {
            ("withdrawal", Some(amount)) => Ok((
                record.client,
                Withdrawal {
                    id: record.tx,
                    amount,
                },
            )),
            ("deposit", Some(amount)) => Ok((
                record.client,
                Deposit {
                    id: record.tx,
                    amount,
                },
            )),
            ("dispute", None) => Ok((record.client, Dispute { id: record.tx })),
            ("resolve", None) => Ok((record.client, Resolve { id: record.tx })),
            ("chargeback", None) => Ok((record.client, Chargeback { id: record.tx })),
            _ => Err(format!("transaction is not valid {:?}", record)),
        }
    }
}

fn process(path: &str, engine: &mut Engine) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    // stops on error
    for result in rdr.deserialize() {
        let record: TransactionRaw = result?;
        let (tid, t) = Transaction::from(record)?;
        engine.process(tid, t);
    }
    Ok(())
}

fn dump_state(engine: &Engine) -> Result<(), Box<dyn Error>> {
    let mut wrtr = csv::Writer::from_writer(io::stdout());

    engine
        .accounts
        .iter()
        .map(AccountStorage::from)
        .map(|a| wrtr.serialize(a))
        .collect::<Result<Vec<_>, _>>()?;

    wrtr.flush()?;

    Ok(())
}

fn main() {
    if let Some(transactions_path) = args().skip(1).next() {
        let mut engine = Engine::empty();

        process(&transactions_path, &mut engine)
            .expect("failed processing transactions");

        dump_state(&engine)
            .expect("failed saving data");
    }
}
