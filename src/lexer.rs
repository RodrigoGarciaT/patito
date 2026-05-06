// Analizador léxico del lenguaje Patito 

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")] // descarta espacios, tabs y saltos de línea
pub enum Token {
    // Palabras reservadas 
    #[token("programa")] Programa,
    #[token("inicio")]   Inicio,
    #[token("fin")]      Fin,
    #[token("vars")]     Vars,
    #[token("entero")]   Entero,
    #[token("flotante")] Flotante,
    #[token("nula")]     Nula,
    #[token("si")]       Si,
    #[token("sino")]     Sino,   // debe declararse antes de Si para preferir "sino" sobre "si"+"no"
    #[token("mientras")] Mientras,
    #[token("haz")]      Haz,
    #[token("escribe")]  Escribe,
    #[token("regresa")]  Regresa,

    // ── Identificadores ───────────────────────────────────────────────────────
    // Empieza con letra, continúa con letras, dígitos o guión bajo.
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Id(String),

    // ── Constantes numéricas ──────────────────────────────────────────────────
    // CteFlot DEBE estar antes de CteEnt: "3.14" tiene más caracteres que "3"
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap())]
    CteFlot(f64),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    CteEnt(i64),

    // ── Cadena de texto (letrero) ─────────────────────────────────────────────
    // Comillas dobles con cualquier carácter excepto comilla y salto de línea.
    #[regex(r#""[^"\n]*""#, |lex| lex.slice().to_owned())]
    Letrero(String),

    // ── Operadores ────────────────────────────────────────────────────────────
    // Los de dos caracteres (==, !=) se declaran antes que los de uno (=, <, >)
    // para que logos los prefiera cuando la entrada tenga ambos caracteres.
    #[token("==")] Igual,
    #[token("!=")] Dif,
    #[token("=")]  Asig,
    #[token("<")]  Menor,
    #[token(">")]  Mayor,
    #[token("+")]  Mas,
    #[token("-")]  Menos,
    #[token("*")]  Por,
    #[token("/")]  Div,

    //Puntuación
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LlaveIzq,
    #[token("}")] LlaveDer,
    #[token("[")] CorcIzq,
    #[token("]")] CorcDer,
    #[token(":")] DosPuntos,
    #[token(";")] PuntoComa,
    #[token(",")] Coma,
}

//  Error léxico

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub position: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "carácter no reconocido en posición {}", self.position)
    }
}

// Adaptador logos a LALRPOP 

/// LALRPOP espera un iterador de `Result<(inicio, Token, fin), Error>`.
pub struct Lexer<'input> {
    inner: logos::Lexer<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer { inner: Token::lexer(input) }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token, usize), LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok  = self.inner.next()?;      // None = fin de la entrada
        let span = self.inner.span();
        match tok {
            Ok(t)  => Some(Ok((span.start, t, span.end))),
            Err(_) => Some(Err(LexError { position: span.start })),
        }
    }
}
