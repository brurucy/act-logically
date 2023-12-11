use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub enum TypedValue {
    Str(String),
    Int(usize),
    Bool(bool),
}

impl Display for TypedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedValue::Str(x) => std::fmt::Display::fmt(&x, f),
            TypedValue::Int(x) => std::fmt::Display::fmt(&x, f),
            TypedValue::Bool(x) => std::fmt::Display::fmt(&x, f),
        }
    }
}

impl Debug for TypedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedValue::Str(x) => std::fmt::Debug::fmt(&x, f),
            TypedValue::Int(x) => std::fmt::Debug::fmt(&x, f),
            TypedValue::Bool(x) => std::fmt::Debug::fmt(&x, f),
        }
    }
}

impl From<String> for TypedValue {
    fn from(value: String) -> Self {
        TypedValue::Str(value)
    }
}

impl From<&str> for TypedValue {
    fn from(value: &str) -> Self {
        TypedValue::Str(value.to_string())
    }
}

impl From<usize> for TypedValue {
    fn from(value: usize) -> Self {
        TypedValue::Int(value)
    }
}

impl From<bool> for TypedValue {
    fn from(value: bool) -> Self {
        TypedValue::Bool(value)
    }
}

pub type Variable = String;

// Unsafe
pub type SkolemFunctionCall = fn(args: HashMap<&str, &TypedValue>) -> TypedValue;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct SkolemFunction {
    pub func: SkolemFunctionCall,
    pub deps: Vec<String>
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub enum Term {
    Variable(String),
    Constant(TypedValue),
    Skolemizer(SkolemFunction),
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Variable(x) => std::fmt::Display::fmt(&x,f),
            Term::Constant(x) => std::fmt::Display::fmt(&x,f),
            Term::Skolemizer(x) => std::fmt::Debug::fmt(x, f)
        }
    }
}

pub type AnonymousGroundAtom = Vec<TypedValue>;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct Atom {
    pub terms: Vec<Term>,
    pub symbol: String,
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", &self.symbol)?;

        for (index, term) in self.terms.iter().enumerate() {
            write!(f, "{}", term)?;
            // Add comma between terms, but not after the last term
            if index < self.terms.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}(", &self.symbol)?;

        for (index, term) in self.terms.iter().enumerate() {
            write!(f, "{:?}", term)?;
            // Add comma between terms, but not after the last term
            if index < self.terms.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}

pub enum Matcher {
    Any,
    Constant(TypedValue),
}

pub struct Query<'a> {
    pub matchers: Vec<Matcher>,
    pub symbol: &'a str,
}

pub struct QueryBuilder<'a> {
    pub query: Query<'a>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(relation: &'a str) -> Self {
        QueryBuilder {
            query: Query {
                matchers: vec![],
                symbol: relation,
            },
        }
    }
    pub fn with_any(&mut self) {
        self.query.matchers.push(Matcher::Any);
    }
    pub fn with_constant(&mut self, value: TypedValue) {
        self.query.matchers.push(Matcher::Constant(value))
    }
}

impl<'a> From<QueryBuilder<'a>> for Query<'a> {
    fn from(value: QueryBuilder<'a>) -> Self {
        value.query
    }
}

#[macro_export]
macro_rules! build_query {
    ($relation:ident ( $( $matcher:tt ),* $(,)? )) => {{
        let mut builder = QueryBuilder::new(stringify!($relation));
        $(
            build_query!(@matcher builder, $matcher);
        )*
        builder.query
    }};
    (@matcher $builder:expr, _) => {{
        $builder.with_any();
    }};
    (@matcher $builder:expr, $value:expr) => {{
        $builder.with_constant($value.into());
    }};
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Atom>,
    pub id: usize,
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.head)?;
        write!(f, " <- [")?;
        for (index, atom) in self.body.iter().enumerate() {
            write!(f, "{}", atom)?;
            if index < self.body.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.head)?;
        write!(f, " <- [")?;
        for (index, atom) in self.body.iter().enumerate() {
            write!(f, "{:?}", atom)?;
            if index < self.body.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Program {
    pub inner: Vec<Rule>,
}

impl From<Vec<Rule>> for Program {
    fn from(value: Vec<Rule>) -> Self {
        let mut val = value;
        val.sort();
        // Questionable, I know :)
        for (id, rule) in val.iter_mut().enumerate() {
            (*rule).id = id;
        }

        Self { inner: val }
    }
}
