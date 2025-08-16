#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    With(String),
    Without(String),
    Changed(String),
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::With(comp) => write!(f, "With<{comp}>"),
            Filter::Without(comp) => write!(f, "Without<{comp}>"),
            Filter::Changed(comp) => write!(f, "Changed<{comp}>"),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Filters(pub(crate) Vec<Filter>);

impl Filters {
    pub fn to_filter_bundle(&self) -> String {
        let content = self
            .0
            .iter()
            .map(Filter::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        if self.0.len() > 1 {
            format!("({})", content)
        } else {
            format!("{}", content)
        }
    }

    pub fn single(filter: Filter) -> Self {
        Filters(vec![filter])
    }

    pub fn with(&mut self, filter: Filter) -> &mut Self {
        self.0.push(filter);
        self
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for Filters {
    fn from(input: &str) -> Self {
        let input = if input.starts_with('{') && input.ends_with('}') {
            &input[1..input.len() - 1]
        } else {
            input
        };

        let filters = input.split(',');
        Filters(
            filters
                .into_iter()
                .map(|filter| {
                    let filter = filter.trim();
                    return if filter.starts_with('!') {
                        Filter::Without(filter[1..].to_string())
                    } else {
                        Filter::With(filter.to_string())
                    };
                })
                .collect(),
        )
    }
}
