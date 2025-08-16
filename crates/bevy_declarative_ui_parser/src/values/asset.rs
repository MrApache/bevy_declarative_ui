use crate::values::parse_params;

#[derive(Clone, PartialEq, Debug)]
pub struct Asset {
    pub path: String,
}

impl Asset {
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

        println!();
        Asset {
            path: params.get("Path").unwrap().to_string(),
        }
    }
}
