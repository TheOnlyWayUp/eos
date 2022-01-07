use std::fmt::Write;

#[cfg(feature = "localtime")]
use crate::sys::localtime;

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(doc)]
use crate::Time;

use crate::{utils::ensure_in_range, DateTime, Error};

/// Represents an offset from UTC.
///
/// This struct can only store values up to ±23:59:59.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset {
    hours: i8,
    minutes: i8,
    seconds: i8,
}

impl Default for UtcOffset {
    fn default() -> Self {
        Self::UTC
    }
}

impl UtcOffset {
    /// Returns the smallest possible [`UtcOffset`].
    pub const MIN: Self = Self {
        hours: -23,
        minutes: -59,
        seconds: -59,
    };

    /// Returns the largest possible [`UtcOffset`].
    pub const MAX: Self = Self {
        hours: 23,
        minutes: 59,
        seconds: 59,
    };

    /// Returns the [`UtcOffset`] representing UTC.
    pub const UTC: Self = Self {
        hours: 0,
        minutes: 0,
        seconds: 0,
    };

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    #[inline]
    pub const fn __new_unchecked_from_macro(hours: i8, minutes: i8, seconds: i8) -> Self {
        Self {
            hours,
            minutes,
            seconds,
        }
    }

    /// Creates a new [`UtcOffset`] from the given number of hours, minutes, and seconds.
    ///
    /// The sign of all three components should match. If they do not, all components will
    /// have their signs flipped to match the `hour` sign.
    ///
    /// The values must be within the range of ±23:59:59.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert!(UtcOffset::from_hms(24, 0, 0).is_err()); // invalid range
    /// assert_eq!(UtcOffset::from_hms(23, 56, 59)?.into_hms(), (23, 56, 59));
    /// assert_eq!(UtcOffset::from_hms(0, 30, 0)?.into_hms(), (0, 30, 0));
    /// assert_eq!(UtcOffset::from_hms(0, -30, 30)?.into_hms(), (0, -30, -30));
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub const fn from_hms(hours: i8, mut minutes: i8, mut seconds: i8) -> Result<Self, Error> {
        ensure_in_range!(hours, -23 => 23);
        ensure_in_range!(minutes, -59 => 59);
        ensure_in_range!(seconds, -59 => 59);

        // This is surprisingly well optimised
        if hours.is_negative() {
            if minutes.is_positive() {
                minutes = -minutes;
            }
            if seconds.is_positive() {
                seconds = -seconds;
            }
        } else if hours.is_positive() {
            if minutes.is_negative() {
                minutes = -minutes;
            }
            if seconds.is_negative() {
                seconds = -seconds;
            }
        } else {
            // Special case for 0 hours, it takes the sign of minutes
            // -30:30 => -30:-30
            // 30:-30 => 30:30
            if seconds.is_positive() != minutes.is_positive() {
                seconds = -seconds;
            }
        }

        Ok(Self {
            hours,
            minutes,
            seconds,
        })
    }

    /// Creates a new [`UtcOffset`] from a total number of seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert_eq!(UtcOffset::from_seconds(23400)?.into_hms(), (6, 30, 0));
    /// assert_eq!(UtcOffset::from_seconds(23400)?.total_seconds(), 23400);
    /// # Ok::<_, eos::Error>(())
    /// ```
    pub const fn from_seconds(seconds: i32) -> Result<Self, Error> {
        ensure_in_range!(seconds, -86399 => 86399);
        Ok(Self::from_seconds_unchecked(seconds))
    }

    pub(crate) const fn from_seconds_unchecked(seconds: i32) -> Self {
        let hours = seconds / 3600;
        let seconds = seconds % 3600;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        Self {
            hours: hours as i8,
            minutes: minutes as i8,
            seconds: seconds as i8,
        }
    }

    /// Returns the total number of seconds this offset represents.
    ///
    /// # Example
    ///
    /// ```
    /// # use eos::UtcOffset;
    /// assert_eq!(UtcOffset::from_hms(6, 30, 0)?.total_seconds(), 23400);
    /// # Ok::<_, eos::Error>(())
    /// ```
    #[inline]
    pub const fn total_seconds(&self) -> i32 {
        self.hours as i32 * 3600 + self.minutes as i32 * 60 + self.seconds as i32
    }

    /// Unwraps this offset into their individual `(hours, minutes, seconds)` components.
    #[inline]
    pub const fn into_hms(self) -> (i8, i8, i8) {
        (self.hours, self.minutes, self.seconds)
    }

    /// Returns `true` if this offset is UTC.
    #[inline]
    pub const fn is_utc(&self) -> bool {
        self.hours == 0 && self.minutes == 0 && self.seconds == 0
    }

    /// Returns `true` if this offset is negative.
    #[inline]
    pub const fn is_negative(&self) -> bool {
        self.hours < 0 && self.minutes < 0 && self.seconds < 0
    }

    /// Subtracts two offsets, returning [`Error`] if the result would be out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east = utc_offset!(-5:00);
    /// let west = utc_offset!(-8:00);
    /// let far  = utc_offset!(18:00);
    ///
    /// assert!(far.checked_sub(west).is_err()); // 18 - -8 => 26
    /// assert_eq!(west.checked_sub(east), Ok(utc_offset!(-3:00)));
    /// ```
    #[inline]
    pub const fn checked_sub(self, other: Self) -> Result<Self, Error> {
        let seconds = self.total_seconds() - other.total_seconds();
        Self::from_seconds(seconds)
    }

    /// Adds two offsets, returning [`Error`] if the result would be out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east  = utc_offset!(-5:00);
    /// let west  = utc_offset!(-8:00);
    /// let far   = utc_offset!(18:00);
    /// let other = utc_offset!(-18:00);
    ///
    /// assert_eq!(far.checked_add(west), Ok(utc_offset!(10:00)));
    /// assert_eq!(west.checked_add(east), Ok(utc_offset!(-13:00)));
    /// assert!(other.checked_add(west).is_err());
    /// assert_eq!(other.checked_add(east), Ok(utc_offset!(-23:00)));
    /// ```
    #[inline]
    pub const fn checked_add(self, other: Self) -> Result<Self, Error> {
        let seconds = self.total_seconds() + other.total_seconds();
        Self::from_seconds(seconds)
    }

    /// Subtracts two offsets, saturating at the bounds if out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east = utc_offset!(-5:00);
    /// let west = utc_offset!(-8:00);
    /// let far  = utc_offset!(18:00);
    ///
    /// assert_eq!(far.saturating_sub(west), utc_offset!(23:59:59)); // 18 - -8 => 26
    /// assert_eq!(west.saturating_sub(east), utc_offset!(-3:00));
    /// ```
    #[inline]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let seconds = self.total_seconds() - other.total_seconds();
        if seconds <= -86399 {
            Self::MIN
        } else if seconds >= 86399 {
            Self::MAX
        } else {
            Self::from_seconds_unchecked(seconds)
        }
    }

    /// Adds two offsets, saturating at the bounds if out of bounds.
    ///
    /// ```rust
    /// # use eos::utc_offset;
    /// let east  = utc_offset!(-5:00);
    /// let west  = utc_offset!(-8:00);
    /// let far   = utc_offset!(18:00);
    /// let other = utc_offset!(-18:00);
    ///
    /// assert_eq!(far.saturating_add(west), utc_offset!(10:00));
    /// assert_eq!(west.saturating_add(east), utc_offset!(-13:00));
    /// assert_eq!(other.saturating_add(west), utc_offset!(-23:59:59));
    /// assert_eq!(other.saturating_add(east), utc_offset!(-23:00));
    /// ```
    #[inline]
    pub const fn saturating_add(self, other: Self) -> Self {
        let seconds = self.total_seconds() + other.total_seconds();
        if seconds <= -86399 {
            Self::MIN
        } else if seconds >= 86399 {
            Self::MAX
        } else {
            Self::from_seconds_unchecked(seconds)
        }
    }
}

impl core::fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hours.fmt(f)?;
        f.write_char(':')?;
        let (m, s) = (self.minutes.abs(), self.seconds.abs());
        m.fmt(f)?;
        if s > 0 {
            f.write_char(':')?;
            s.fmt(f)?;
        }
        Ok(())
    }
}

impl core::ops::Neg for UtcOffset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            hours: -self.hours,
            minutes: -self.minutes,
            seconds: -self.seconds,
        }
    }
}

/// A trait that defines timezone behaviour.
pub trait TimeZone: Clone {
    /// Returns the name of the timezone at a given datetime.
    #[cfg(feature = "alloc")]
    fn name<Tz: TimeZone>(&self, _datetime: &DateTime<Tz>) -> Option<String> {
        None
    }

    /// Returns the UTC offset of the timezone at a given datetime.
    ///
    /// If DST is being observed then the offset must take that into account.
    fn offset<Tz: TimeZone>(&self, datetime: &DateTime<Tz>) -> UtcOffset;

    /// Returns the DST offset if it's being observed.
    ///
    /// Note that despite this method existing, [`TimeZone::offset`] must
    /// still include this offset. This method is mainly used in aiding for
    /// timezone movement calculations.
    ///
    /// If DST is not being observed for this TimeZone at the given date
    /// then [`UtcOffset::UTC`] should be returned.
    fn dst_offset<Tz: TimeZone>(&self, datetime: &DateTime<Tz>) -> UtcOffset;

    /// Converts from a UTC [`DateTime`] to a datetime in this timezone.
    fn datetime_at(self, mut utc: DateTime<Utc>) -> DateTime<Self>
    where
        Self: Sized,
    {
        // Algorithm taken from the PSF
        let offset = self.offset(&utc);
        let mut dst = self.dst_offset(&utc);
        if let Ok(delta) = offset.checked_sub(dst) {
            if !delta.is_utc() {
                utc.shift(delta);
                dst = self.dst_offset(&utc);
            }
        }
        utc.shift(dst);
        utc.with_timezone(self)
    }
}

impl TimeZone for UtcOffset {
    fn offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        *self
    }

    fn dst_offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        UtcOffset::UTC
    }
}

/// Represents the UTC timezone.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;

impl TimeZone for Utc {
    #[cfg(feature = "alloc")]
    fn name<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> Option<String> {
        Some(String::from("UTC"))
    }

    fn offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        UtcOffset::UTC
    }

    fn dst_offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        UtcOffset::UTC
    }
}

impl Utc {
    /// Returns the current [`DateTime`] in UTC.
    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn now() -> DateTime<Self> {
        DateTime::utc_now()
    }
}

/// Represents the machine's local timezone.
///
/// Due to differences in operating systems, the information returned by this
/// struct isn't necessarily the most detailed.
///
/// This requires the `localtime` feature to be enabled.
///
/// # Underlying OS APIs
///
/// Currently, the following OS APIs are being used to get the local timezone:
///
/// | Platform |                  Function Call                  |
/// |----------|-------------------------------------------------|
/// | POSIX    | [`localtime_r`]                                 |
/// | Windows  | [`GetTimeZoneInformation`] and [`GetLocalTime`] |
///
/// **Disclaimer**: These OS APIs might change over time.
///
/// [`localtime_r`]: https://linux.die.net/man/3/localtime_r
/// [`GetTimeZoneInformation`]: https://docs.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation
/// [`GetLocalTime`]: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
///
/// ## Why `localtime_r`?
///
/// Users familiar with the [`chrono`] or the [`time`] crate would be aware that this underlying C API has been
/// the [source] of an CVE ([CVE-2020-26235][CVE]). However, as more time has passed on the issue there are dissenting
/// views within both the Rust community and the C community on whether the issue lied within `localtime_r` or `setenv`. This
/// is because the `localtime_r` function only *reads* from the environment, it does not modify it. Many users have
/// expressed the opinion that the issue is the underlying [`std::env::set_env`] call not being marked `unsafe`
/// since by its nature it is not a thread safe function. There have been many proposals that have gained significant
/// traction to mark `set_env` as `unsafe`. Given the direction the standard library is heading, the opinion of various
/// experts, and the fact that there is no feasible alternative API in POSIX, this library opts to remain using
/// `localtime_r` despite its previously tainted status causing an CVE.
///
/// If an alternative API were to exist that would fill this gap of functionality for POSIX systems the library gladly
/// switch to it, however, this ideal has yet to be met.
///
/// [`chrono`]: https://github.com/chronotope/chrono/
/// [`time`]: https://github.com/time-rs/time/
/// [source]: https://passcod.name/technical/no-time-for-chrono.html
/// [CVE]: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2020-26235
///
#[cfg(feature = "localtime")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Local(pub(crate) localtime::LocalTime);

impl core::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "alloc") {
            f.debug_struct("Local")
                .field("offset", &self.0.offset())
                .field("name", &self.0.name())
                .finish()
        } else {
            f.debug_struct("Local").field("offset", &self.0.offset()).finish()
        }
    }
}

#[cfg(feature = "localtime")]
impl Local {
    /// Creates a new [`Local`].
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self(localtime::LocalTime::new()?))
    }

    /// Returns the current [`DateTime`] in local time.
    #[inline]
    pub fn now() -> Result<DateTime<Self>, Error> {
        DateTime::now()
    }
}

impl TimeZone for Local {
    #[cfg(feature = "alloc")]
    fn name<Tz: TimeZone>(&self, _datetime: &DateTime<Tz>) -> Option<String> {
        self.0.name()
    }

    fn offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        self.0.offset()
    }

    fn dst_offset<Tz: TimeZone>(&self, _: &DateTime<Tz>) -> UtcOffset {
        self.0.dst_offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_ranges() {
        assert!(UtcOffset::from_hms(-32, 0, 0).is_err());
        assert!(UtcOffset::from_hms(24, 0, 0).is_err());
        assert!(UtcOffset::from_hms(23, 60, 0).is_err());
        assert!(UtcOffset::from_hms(-23, -60, 0).is_err());
        assert!(UtcOffset::from_hms(-23, -60, -60).is_err());
        assert!(UtcOffset::from_hms(24, -60, -60).is_err());

        assert!(UtcOffset::from_hms(-5, 30, 0).is_ok());

        assert!(UtcOffset::from_seconds(-86400).is_err());
        assert!(UtcOffset::from_seconds(86400).is_err());
        assert!(UtcOffset::from_seconds(3600).is_ok());
        assert!(UtcOffset::from_seconds(-3600).is_ok());
    }
}
