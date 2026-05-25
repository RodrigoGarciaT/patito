#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Entero,
    Flotante,
    Nula,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Entero   => write!(f, "entero"),
            Type::Flotante => write!(f, "flotante"),
            Type::Nula     => write!(f, "nula"),
        }
    }
}
