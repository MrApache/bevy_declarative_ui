use std::collections::HashMap;

pub(crate) trait IsCurlyBracesEnclosed {
    fn is_curly_braces_enclosed(&self) -> bool;
}

impl IsCurlyBracesEnclosed for &str {
    fn is_curly_braces_enclosed(&self) -> bool {
        self.starts_with('{') && self.ends_with('}')
    }
}

impl IsCurlyBracesEnclosed for String {
    fn is_curly_braces_enclosed(&self) -> bool {
        self.as_str().is_curly_braces_enclosed()
    }
}

impl<V> GetOrInsertEmpty<V> for HashMap<String, V>
where
    V: Default,
{
    fn get_or_insert_empty(&mut self, key: String) -> &mut V {
        self.entry(key).or_default()
    }

    fn get_or_insert<F: Fn() -> V>(&mut self, key: &str, value: F) -> &mut V {
        if !self.contains_key(key) {
            self.insert(key.to_string(), value());
        }

        self.get_mut(key).unwrap()
    }
}

pub trait GetOrInsertEmpty<V> {
    fn get_or_insert_empty(&mut self, key: String) -> &mut V
    where
        V: Default;

    fn get_or_insert<F>(&mut self, key: &str, value: F) -> &mut V
    where
        F: Fn() -> V;
}

pub trait TrimExtension {
    fn trim_ext(&self) -> TrimResult;
}

pub struct TrimResult<'a> {
    pub before: usize,
    pub trimmed_after: usize,
    pub string: &'a str,
}

impl TrimExtension for str {
    fn trim_ext(&self) -> TrimResult {
        let before = self.trim_start();
        let after = before.trim_end();
        TrimResult {
            before: self.len() - before.len(),
            trimmed_after: before.len() - after.len(),
            string: after,
        }
    }
}

impl TrimExtension for String {
    fn trim_ext(&self) -> TrimResult {
        self.as_str().trim_ext()
    }
}
