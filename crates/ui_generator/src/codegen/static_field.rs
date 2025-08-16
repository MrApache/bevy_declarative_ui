use std::fmt::{Display, Formatter};

pub struct StaticField {
    name: String,
    r#type: String,
    default: String,
}

impl StaticField {
    pub fn new(
        name: impl Into<String>,
        r#type: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            default: default.into(),
        }
    }
}

impl Display for StaticField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "static {0}: {1} = {2};",
            self.name, self.r#type, self.default
        )
    }
}
