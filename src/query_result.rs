use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct RXQLiteQueryResult {
    pub(super) changes: u64,
    pub(super) last_insert_rowid: i64,
}

impl RXQLiteQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.changes
    }

    pub fn last_insert_rowid(&self) -> i64 {
        self.last_insert_rowid
    }
}

impl Extend<RXQLiteQueryResult> for RXQLiteQueryResult {
    fn extend<T: IntoIterator<Item = RXQLiteQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.changes += elem.changes;
            self.last_insert_rowid = elem.last_insert_rowid;
        }
    }
}
