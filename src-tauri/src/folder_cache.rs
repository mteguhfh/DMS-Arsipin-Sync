use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FolderCache {
    path_to_id: HashMap<String, String>,
    id_to_path: HashMap<String, String>,
}

impl FolderCache {
    pub fn new() -> Self {
        Self {
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: &str, id: &str) {
        self.path_to_id.insert(path.to_string(), id.to_string());
        self.id_to_path.insert(id.to_string(), path.to_string());
    }

    pub fn get_id(&self, path: &str) -> Option<&String> {
        self.path_to_id.get(path)
    }

    pub fn get_path(&self, id: &str) -> Option<&String> {
        self.id_to_path.get(id)
    }

    pub fn remove_by_path(&mut self, path: &str) {
        if let Some(id) = self.path_to_id.remove(path) {
            self.id_to_path.remove(&id);
        }
    }

    pub fn from_pairs(pairs: &[(String, String)]) -> Self {
        let mut cache = Self::new();
        for (path, id) in pairs {
            cache.insert(path, id);
        }
        cache
    }

    pub fn to_pairs(&self) -> Vec<(String, String)> {
        self.path_to_id
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn contains_path(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }

    pub fn is_empty(&self) -> bool {
        self.path_to_id.is_empty()
    }
}
