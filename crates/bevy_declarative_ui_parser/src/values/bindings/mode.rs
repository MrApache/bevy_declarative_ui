#[derive(Copy, Clone, Default, Debug, PartialEq, Hash, Eq)]
pub enum BindingMode {
    Read,
    Write,
    ReadWrite,
    #[default]
    ReadOnce,
}

impl BindingMode {
    pub fn from_str(value: &str) -> Self {
        match value {
            "Read"      => Self::Read,
            "Write"     => Self::Write,
            "ReadWrite" => Self::ReadWrite,
            "ReadOnce"  => Self::ReadOnce,
            _ => panic!(),
        }
    }
}