use serde::Deserialize;
use wasm_bindgen::JsCast;

/// A single entry in the virtual filesystem (file or directory).
#[derive(Debug, Clone, Deserialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub children: Option<Vec<FsEntry>>,
}

/// Represents the virtual filesystem loaded from content/filesystem.json.
/// Holds the root-level directory entries.
#[derive(Debug, Clone)]
pub struct VirtualFS {
    root: Vec<FsEntry>,
}

impl VirtualFS {
    /// Fetch and parse content/filesystem.json, returning a VirtualFS.
    pub async fn load() -> Self {
        use wasm_bindgen_futures::JsFuture;

        let window = web_sys::window().expect("no window");
        let promise = window.fetch_with_str("content/filesystem.json");
        let resp_val = JsFuture::from(promise)
            .await
            .expect("failed to fetch filesystem.json");
        let resp: web_sys::Response = resp_val
            .dyn_into()
            .expect("fetch result is not a Response");

        let text_promise = resp.text().expect("failed to get response text");
        let text_val = JsFuture::from(text_promise)
            .await
            .expect("failed to get text from response");
        let text = text_val
            .as_string()
            .expect("response text is not a string");

        let root: Vec<FsEntry> =
            serde_json::from_str(&text).expect("failed to parse filesystem.json");

        VirtualFS { root }
    }

    /// Return the children of the directory at `path`.
    /// `"/"` or `""` returns the root entries.
    /// Returns `None` if the path does not exist or is not a directory.
    pub fn list_children(&self, path: &str) -> Option<&[FsEntry]> {
        if path == "/" || path.is_empty() {
            return Some(&self.root);
        }
        let entry = self.find_entry(path)?;
        if entry.entry_type != "dir" {
            return None;
        }
        entry.children.as_deref()
    }

    /// Return the content path for a file entry. Returns `None` if not found
    /// or if the entry is a directory.
    pub fn get_file_path(&self, path: &str) -> Option<String> {
        let entry = self.find_entry(path)?;
        if entry.entry_type == "file" {
            Some(entry.path.clone())
        } else {
            None
        }
    }

    // ---- internal helpers ----

    fn find_entry(&self, path: &str) -> Option<&FsEntry> {
        Self::find_in_entries(&self.root, path)
    }

    fn find_in_entries<'a>(entries: &'a [FsEntry], path: &str) -> Option<&'a FsEntry> {
        for entry in entries {
            if entry.path == path {
                return Some(entry);
            }
            if let Some(children) = &entry.children {
                if let found @ Some(_) = Self::find_in_entries(children, path) {
                    return found;
                }
            }
        }
        None
    }
}
