use std::ops::Deref;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// ArchVecs represents a Vector of possibly architecture specific fields and their values.
///
/// ArchVecs holds a list of architectures and associated values. Earch architecture holds one to
/// many values. The same values may appear across different architectures. An architectures of None
/// indicates that the value applies to all architectures.
///
/// When working with ArchVecs you generally want to take the values relevant to you.
/// This is simply the list of values belonging to your set architecture plus
/// the values with a None architecture. [`ArchVecs::arch`] provides this
/// functionality.
pub struct ArchVecs {
    /// A vector of each architecture and their values
    pub(crate) vecs: Vec<ArchVec>,
}

impl Deref for ArchVecs {
    type Target = [ArchVec];

    fn deref(&self) -> &Self::Target {
        &self.vecs
    }
}

impl<'a> IntoIterator for &'a ArchVecs {
    type IntoIter = <&'a Vec<ArchVec> as IntoIterator>::IntoIter;
    type Item = &'a ArchVec;

    fn into_iter(self) -> Self::IntoIter {
        self.vecs.iter()
    }
}

impl From<Vec<ArchVec>> for ArchVecs {
    fn from(vecs: Vec<ArchVec>) -> Self {
        ArchVecs { vecs }
    }
}

impl ArchVecs {
    /// Creates a new empty ArchVecs
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the list of values that have the specified architecture
    pub fn get_any(&self) -> Option<&ArchVec> {
        self.vecs
            .iter()
            .find(|v| v.arch() == None)
    }

    /// Gets the list of values that have the specified architecture
    pub fn get<S: AsRef<str>>(&self, arch: Option<S>) -> Option<&ArchVec> {
        self.vecs
            .iter()
            .find(|v| v.arch() == arch.as_ref().map(|a| a.as_ref()))
    }

    /// Gets the list of values that apply to the given architecture
    ///
    /// The returned values with either belong to the given architecture or the None architecture.
    pub fn arch<S: AsRef<str>>(&self, arch: S) -> impl Iterator<Item = &str> {
        self.vecs
            .iter()
            .filter(move |v| v.supports(arch.as_ref()))
            .flatten()
    }

    /// Returns an iterator over all values in this ArchVecs
    ///
    /// You usually don't want this function as it means architecture specific fields are not being
    /// handled.
    pub fn all(&self) -> impl Iterator<Item = &str> {
        self.vecs.iter().flatten()
    }

    /// Gets the list of values that have no specific architecture
    pub fn any(&self) -> impl Iterator<Item = &str> {
        self.get_any()
            .map(|v| v.iter())
            .unwrap_or_default()
    }
}

/// ArchVec represents a possibly architecture specific field and its values.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchVec {
    /// The architecture of the field, None is equivalent to 'any'
    pub(crate) arch: Option<String>,
    /// The values the field contains
    pub(crate) values: Vec<String>,
}

impl Default for &ArchVec {
    fn default() -> Self {
        static EMPTY: ArchVec = ArchVec { arch: None, values: Vec::new() };
        &EMPTY
    }
}

/// Iterator over ArchVec values
#[derive(Clone, Debug, Default)]
pub struct ArchVecIter<'a>(std::slice::Iter<'a, String>);

impl<'a> Iterator for ArchVecIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(String::as_str)
    }
}

impl<'a> IntoIterator for &'a ArchVec {
    type IntoIter = ArchVecIter<'a>;
    type Item = &'a str;

    fn into_iter(self) -> Self::IntoIter {
        ArchVecIter(self.values.iter())
    }
}

impl From<String> for ArchVec {
    fn from(value: String) -> Self {
        ArchVec::new(Some(value))
    }
}

impl From<&str> for ArchVec {
    fn from(value: &str) -> Self {
        ArchVec::new(Some(value))
    }
}

impl From<Option<String>> for ArchVec {
    fn from(value: Option<String>) -> Self {
        ArchVec::new(value)
    }
}

impl From<Option<&str>> for ArchVec {
    fn from(value: Option<&str>) -> Self {
        ArchVec::new(value)
    }
}

impl ArchVec {
    /// Creates a new ArchVec with the given architecture
    pub fn new<S: Into<String>>(arch: Option<S>) -> ArchVec {
        let arch = arch.map(|x| x.into());
        ArchVec {
            arch,
            values: Vec::new(),
        }
    }

    /// Creates a new ArchVec with the given architecture and values
    pub fn with_values<S: Into<String>>(arch: Option<S>, values: Vec<String>) -> ArchVec {
        let arch = arch.map(|x| x.into());
        ArchVec { arch, values }
    }

    /// An iterator over the values in this ArchVec
    pub fn iter(&self) -> ArchVecIter<'_> {
        self.into_iter()
    }

    /// A list of the ArchVec's values
    pub fn values(&self) -> &[String] {
        &self.values
    }

    /// Gets the architecture
    pub fn arch(&self) -> Option<&str> {
        self.arch.as_deref()
    }

    /// Checks if a given ArchVec is supported by a given architecture
    ///
    /// An ArchVec supports an architecture if the architecture is the same as the ArchVec's or
    /// the ArchVec does not have an architecture.
    pub fn supports<S: AsRef<str>>(&self, s: S) -> bool {
        self.arch().is_none_or(|a| a == s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Srcinfo;

    #[test]
    fn test_archvec() {
        let av = ArchVec::from("x86_64");
        assert!(av.supports("x86_64"));

        let av = ArchVec::default();
        assert!(av.supports("x86_64"));

        let av = ArchVec::from("i686");
        assert!(!av.supports("x86_64"));

        let srcinfo: Srcinfo = include_str!("../tests/srcinfo/libc++").parse().unwrap();
        let depends = srcinfo.makedepends().arch("x86_64").collect::<Vec<_>>();

        let expected = vec!["clang", "cmake", "ninja", "python", "libunwind"];

        assert_eq!(expected, depends);
    }
}
