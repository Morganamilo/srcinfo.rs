use std::ops::Deref;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// ArchVecs represents a Vector of possibly architecture specific fields and their values.
pub struct ArchVecs {
    /// A vector of each architecture and their values
    pub vecs: Vec<ArchVec>,
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
    pub fn get<S: AsRef<str>>(&self, arch: Option<S>) -> Option<&ArchVec> {
        self.vecs
            .iter()
            .find(|v| v.arch() == arch.as_ref().map(|a| a.as_ref()))
    }

    /// Gets the list of values that apply to the given architecture
    ///
    /// The architecture must either equal the given architecture or be None
    pub fn arch<S: AsRef<str>>(&self, arch: S) -> impl Iterator<Item = &str> {
        self.vecs
            .iter()
            .filter(move |v| v.supports(arch.as_ref()))
            .flatten()
    }
}

/// ArchVec represents a possibly architecture specific field and its values.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchVec {
    /// The architecture of the field, None is equivalent to 'any'
    pub arch: Option<String>,
    /// The values the field contains
    pub values: Vec<String>,
}

/// Iterator over ArchVec values
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
        ArchVec::new(Some(value), Vec::new())
    }
}

impl From<&str> for ArchVec {
    fn from(value: &str) -> Self {
        ArchVec::new(Some(value), Vec::new())
    }
}

impl ArchVec {
    /// Creates a new ArchVec
    pub fn new<S: Into<String>>(arch: Option<S>, vec: Vec<String>) -> ArchVec {
        let arch = arch.map(|x| x.into());
        ArchVec { arch, values: vec }
    }

    /// An iterator over the values in this ArchVec
    pub fn iter(&self) -> ArchVecIter {
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

    /// Checks if a given ArchVec supports a given architecture.
    ///
    /// Returns true if self.arch is none or matches s.
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
