use crate::values::parse_params;

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub path: String,
}

impl Item {
    pub fn parse(params: &str) -> Self {
        let (path, params_str) = match params.split_once(',') {
            Some((p, params)) => (p.trim(), Some(params.trim())),
            None => (params.trim(), None),
        };

        let path = if path.starts_with("Path") {
            path.split_once('=').unwrap().1
        } else {
            path
        };

        let mut params = parse_params(params_str.unwrap_or_default());
        if !path.is_empty() {
            params.insert("Path", path);
        }

        if params.is_empty() {
            panic!("Missing parameters: Path");
        }

        Item {
            path: params.get("Path").unwrap().to_string(),
        }
    }
}
