// Bases de segmentos de memoria virtual (Entrega 4).
// Cada segmento tiene 1000 direcciones reservadas. Patito no tiene booleanos.
pub const GLOBAL_INT_BASE:   u32 = 1000;
pub const GLOBAL_FLOAT_BASE: u32 = 2000;
pub const LOCAL_INT_BASE:    u32 = 5000;
pub const LOCAL_FLOAT_BASE:  u32 = 6000;
pub const TEMP_INT_BASE:     u32 = 13000;
pub const TEMP_FLOAT_BASE:   u32 = 14000;
pub const CTE_INT_BASE:      u32 = 18000;
pub const CTE_FLOAT_BASE:    u32 = 19000;

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
