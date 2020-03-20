use std::fmt;

use ordered_float::OrderedFloat;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Number {
    n: N,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum N {
    Int(i64),
    Float(OrderedFloat<f64>),
    Ref(String),
}

impl Number {
    #[inline]
    pub fn is_i64(&self) -> bool {
        match &self.n {
            N::Int(_) => true,
            N::Float(_) => false,
            N::Ref(_) => self.as_i64().is_some(),
        }
    }

    #[inline]
    pub fn is_f64(&self) -> bool {
        match &self.n {
            N::Float(_) => true,
            N::Int(_) => false,
            N::Ref(ref s) => {
                for c in s.chars() {
                    if c == '.' || c == 'e' || c == 'E' {
                        return s.parse::<f64>().ok().map_or(false, |f| f.is_finite());
                    }
                }
                false
            }
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            N::Int(n) => Some(n),
            N::Float(_) => None,
            N::Ref(ref s) => s.parse().ok(),
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            N::Int(n) => Some(n as f64),
            N::Float(f) => Some(f.into_inner()),
            N::Ref(ref s) => s.parse().ok(),
        }
    }

    #[inline]
    pub fn from_f64(f: f64) -> Option<Number> {
        if f.is_finite() {
            let n = N::Float(OrderedFloat(f));
            Some(Number { n })
        } else {
            None
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.n {
            N::Int(i) => fmt::Display::fmt(&i, formatter),
            N::Float(f) => fmt::Display::fmt(&f, formatter),
            N::Ref(ref s) => fmt::Display::fmt(&s, formatter),
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = formatter.debug_tuple("Number");
        match self.n {
            N::Int(i) => {
                debug.field(&i);
            }
            N::Float(i) => {
                debug.field(&i);
            }
            N::Ref(ref s) => {
                debug.field(&s);
            }
        }
        debug.finish()
    }
}

macro_rules! impl_from_int {
    ( $($ty:ty),* ) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(i: $ty) -> Self {
                    let n = N::Int(i as i64);
                    Number { n }
                }
            }
        )*
    }
}

impl_from_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

macro_rules! impl_from_float {
    ( $($ty:ty),* ) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(f: $ty) -> Self {
                    let n = N::Float(OrderedFloat(f.into()));
                    Number { n }
                }
            }
        )*
    }
}

impl_from_float!(f32, f64);
