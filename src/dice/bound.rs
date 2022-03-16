use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt::{self, Debug},
    ops::{Bound, RangeBounds},
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Bounded {
    start: Bound<i8>,
    end: Bound<i8>,
}

impl Bounded {
    pub fn from_range<T: RangeBounds<i8>>(range: T) -> Self {
        Self::new(range.start_bound().cloned(), range.end_bound().cloned())
    }

    pub fn range_from(from: i8) -> Self {
        Self::new(Bound::Included(from), Bound::Unbounded)
    }

    pub fn range_to(to: i8) -> Self {
        Self::new(Bound::Unbounded, Bound::Included(to))
    }

    // pub fn range_between(from: i8, to: i8) -> Self {
    //     Self::new(Bound::Included(from), Bound::Included(to))
    // }

    // pub fn range_of(value: i8) -> Self {
    //     Self::new(Bound::Included(value), Bound::Included(value))
    // }

    fn new(start: Bound<i8>, end: Bound<i8>) -> Self {
        Self { start, end }
    }

    fn cmp_bound(this: &Bound<i8>, that: &Bound<i8>) -> Ordering {
        match (this, that) {
            (Bound::Unbounded, Bound::Unbounded) => Ordering::Equal,
            (
                Bound::Included(v1) | Bound::Excluded(v1),
                Bound::Included(v2) | Bound::Excluded(v2),
            ) => v1.cmp(&v2),
            (Bound::Included(_) | Bound::Excluded(_), Bound::Unbounded) => Ordering::Less,
            _ => Ordering::Greater,
        }
    }
}

impl RangeBounds<i8> for Bounded {
    fn start_bound(&self) -> Bound<&i8> {
        match self.start {
            Bound::Included(ref start) => Bound::Included(start),
            Bound::Excluded(ref start) => Bound::Excluded(start),
            Bound::Unbounded => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&i8> {
        match self.end {
            Bound::Included(ref end) => Bound::Included(end),
            Bound::Excluded(ref end) => Bound::Excluded(end),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

impl Ord for Bounded {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = Self::cmp_bound(&self.start, &other.start);
        if cmp == Ordering::Equal {
            Self::cmp_bound(&self.end, &other.end)
        } else {
            cmp
        }
    }
}

impl PartialOrd for Bounded {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for Bounded {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}..{}",
            match self.start {
                Bound::Included(value) | Bound::Excluded(value) => value.to_string(),
                Bound::Unbounded => String::from(""),
            },
            match self.end {
                Bound::Included(value) => format!("={}", value),
                Bound::Excluded(value) => value.to_string(),
                Bound::Unbounded => String::from(""),
            }
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        std::{cmp::Ordering::*, ops::Bound::*},
    };

    #[test]
    fn check_compare() {
        assert_eq!(Bounded::cmp_bound(&Unbounded, &Unbounded), Equal);
        assert_eq!(Bounded::cmp_bound(&Unbounded, &Included(10)), Greater);
        assert_eq!(Bounded::cmp_bound(&Unbounded, &Excluded(10)), Greater);
        assert_eq!(Bounded::cmp_bound(&Included(10), &Unbounded), Less);
        assert_eq!(Bounded::cmp_bound(&Included(10), &Included(10)), Equal);
        assert_eq!(Bounded::cmp_bound(&Included(10), &Included(20)), Less);
        assert_eq!(Bounded::cmp_bound(&Included(10), &Excluded(10)), Equal);
        assert_eq!(Bounded::cmp_bound(&Included(10), &Excluded(20)), Less);
        assert_eq!(Bounded::cmp_bound(&Excluded(10), &Unbounded), Less);
        assert_eq!(Bounded::cmp_bound(&Excluded(10), &Included(10)), Equal);
        assert_eq!(Bounded::cmp_bound(&Excluded(10), &Included(20)), Less);
        assert_eq!(Bounded::cmp_bound(&Excluded(10), &Excluded(10)), Equal);
        assert_eq!(Bounded::cmp_bound(&Excluded(10), &Excluded(20)), Less);
    }

    #[test]
    fn check_debug() {
        assert_eq!(format!("{:?}", Bounded::new(Unbounded, Unbounded)), "..");
        assert_eq!(
            format!("{:?}", Bounded::new(Included(10), Unbounded)),
            "10.."
        );
        assert_eq!(
            format!("{:?}", Bounded::new(Excluded(10), Unbounded)),
            "10.."
        );
        assert_eq!(
            format!("{:?}", Bounded::new(Unbounded, Included(20))),
            "..=20"
        );
        assert_eq!(
            format!("{:?}", Bounded::new(Unbounded, Excluded(20))),
            "..20"
        );
        assert_eq!(
            format!("{:?}", Bounded::new(Included(10), Included(20))),
            "10..=20"
        );
        assert_eq!(
            format!("{:?}", Bounded::new(Included(10), Excluded(20))),
            "10..20"
        );
    }
}
