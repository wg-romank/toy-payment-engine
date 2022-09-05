use rust_decimal::Decimal;

// todo: it might be safer to use newtype pattern
// but it adds a lot of boilerplate here so I decided not to
// as long as those are different types and cannot be mixed-up that is
pub type TransactionId = u32;
pub type AccountId = u16;
pub type Currency = Decimal;
