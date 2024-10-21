use std::collections::HashSet;

pub trait EvaluateKeyNamesProvider {
    fn evaluate_key_names(&self) -> Vec<&'static str>;
}

pub struct PrimaryIndex {
    pub hash_key: &'static str,
    pub range_key: Option<&'static str>,
}
impl EvaluateKeyNamesProvider for PrimaryIndex {
    fn evaluate_key_names(&self) -> Vec<&'static str> {
        let mut owned = vec![self.hash_key];
        if let Some(k) = self.range_key {
            owned.push(k);
        }
        owned
    }
}

pub struct SecondaryIndex {
    pub name: &'static str,
    pub hash_key: &'static str,
    pub range_key: Option<&'static str>,
    pub primary_index: &'static PrimaryIndex,
}
impl EvaluateKeyNamesProvider for SecondaryIndex {
    fn evaluate_key_names(&self) -> Vec<&'static str> {
        let mut owned = vec![self.hash_key];
        if let Some(k) = self.range_key {
            owned.push(k);
        }
        owned
            .into_iter()
            .chain(self.primary_index.evaluate_key_names())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}

pub const GENERAL_PRIMARY_INDEX: PrimaryIndex = PrimaryIndex {
    hash_key: "pk",
    range_key: Some("sk"),
};
