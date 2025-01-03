use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::error::Error;
use crate::parse::Parser;

macro_rules! get {
    ($fn:ident, $( $field:ident ).+, $typ:ty) => {
        #[allow(missing_docs)]
        pub fn $fn(&self) -> $typ {
            &self. $( $field ).+
        }
    };
}
macro_rules! geto {
    ($fn:ident, $( $field:ident ).+, $typ:ty) => {
        #[allow(missing_docs)]
        pub fn $fn(&self) -> $typ {
            self. $( $field ).+ .as_deref()
        }
    };
}

/// ArchVec represents a Vector of possibly architecture specific fields.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchVec {
    /// The architecture of the field, None is equivalent to 'any'
    pub arch: Option<String>,
    /// The items the field contains
    pub vec: Vec<String>,
}

impl<S: Into<String>> From<S> for ArchVec {
    fn from(arch: S) -> ArchVec {
        ArchVec {
            arch: Some(arch.into()),
            vec: Vec::new(),
        }
    }
}

impl ArchVec {
    /// Create a new ArchVec from an optional architecture and a vec
    pub fn new<S: Into<String>>(arch: Option<S>, vec: Vec<String>) -> ArchVec {
        let arch = arch.map(|x| x.into());
        ArchVec { arch, vec }
    }

    /// Create a new ArchVec from a given architecture and a vec
    pub fn with_arch<S: Into<String>>(arch: S, vec: Vec<String>) -> ArchVec {
        Self::new(Some(arch), vec)
    }

    /// Create a new ArchVec that allows any architecture
    pub fn any(vec: Vec<String>) -> ArchVec {
        Self::new(Option::<&str>::None, vec)
    }

    /// A list of the ArchVec's values
    pub fn all(&self) -> &[String] {
        &self.vec
    }

    /// Returns a list of values that are active under the given architecture
    pub fn values<S: AsRef<str>>(&self, arch: S) -> &[String] {
        if self.supports(arch) {
            self.all()
        } else {
            &[]
        }
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

    #[doc(hidden)]
    pub fn supported<S: AsRef<str>>(v: &[ArchVec], arch: S) -> impl Iterator<Item = &str> {
        Self::active(v, arch)
    }

    /// Creates an Iterator out of a slice of ArchVecs yielding only entries that would be included
    /// under the given architecture.
    pub fn active<S: AsRef<str>>(v: &[ArchVec], arch: S) -> impl Iterator<Item = &str> {
        v.iter()
            .flat_map(move |v| v.values(arch.as_ref()))
            .map(|s| s.as_str())
    }
}

/// The fields from a .SRCINFO that only apply to the pkgbase.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PackageBase {
    pub pkgbase: String,
    pub pkgver: String,
    pub pkgrel: String,
    pub epoch: Option<String>,
    pub source: Vec<ArchVec>,
    pub valid_pgp_keys: Vec<String>,
    pub no_extract: Vec<String>,
    pub md5sums: Vec<ArchVec>,
    pub sha1sums: Vec<ArchVec>,
    pub sha224sums: Vec<ArchVec>,
    pub sha256sums: Vec<ArchVec>,
    pub sha384sums: Vec<ArchVec>,
    pub sha512sums: Vec<ArchVec>,
    pub b2sums: Vec<ArchVec>,
    pub makedepends: Vec<ArchVec>,
    pub checkdepends: Vec<ArchVec>,
}

/// The fields from a .SRCINFO that are unique to each package.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Package {
    pub pkgname: String,
    pub pkgdesc: Option<String>,
    pub arch: Vec<String>,
    pub url: Option<String>,
    pub license: Vec<String>,
    pub groups: Vec<String>,
    pub depends: Vec<ArchVec>,
    pub optdepends: Vec<ArchVec>,
    pub provides: Vec<ArchVec>,
    pub conflicts: Vec<ArchVec>,
    pub replaces: Vec<ArchVec>,
    pub backup: Vec<String>,
    pub options: Vec<String>,
    pub install: Option<String>,
    pub changelog: Option<String>,
}

impl Package {
    get!(pkgname, pkgname, &str);
    geto!(pkgdesc, pkgdesc, Option<&str>);
    get!(arch, arch, &[String]);
    geto!(url, url, Option<&str>);
    get!(license, license, &[String]);
    get!(groups, groups, &[String]);
    get!(depends, depends, &[ArchVec]);
    get!(optdepends, optdepends, &[ArchVec]);
    get!(provides, provides, &[ArchVec]);
    get!(conflicts, conflicts, &[ArchVec]);
    get!(replaces, replaces, &[ArchVec]);
    get!(backup, backup, &[String]);
    get!(options, options, &[String]);
    geto!(install, install, Option<&str>);
    geto!(changelog, changelog, Option<&str>);
}

/// A complete representation of a .SRCINFO file.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Srcinfo {
    /// The header comment
    pub comment: String,
    /// Fields belonging to the pkgbase
    pub base: PackageBase,
    /// Fields belonging to the pkgbase, may be overridden inside of each package
    pub pkg: Package,
    /// The packages this .SRCINFO contains
    pub pkgs: Vec<Package>,
}

impl FromStr for Srcinfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::parse(s.as_bytes())
    }
}

impl Srcinfo {
    /// Parse a BufRead.
    /// If you are parsing a string directly from_str() should be used instead.
    ///
    /// ```
    /// // from_str() would be better here.
    /// // parse_buf() is only used for the sake of example.
    /// # use srcinfo::Error;
    /// use srcinfo::Srcinfo;
    ///
    /// # fn test() -> Result<(), Error> {
    /// let buf = "
    /// pkgbase = example
    /// pkgver = 1.5.0
    /// pkgrel = 5
    ///
    /// pkgname = example".as_bytes();
    ///
    /// Srcinfo::parse_buf(buf)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_buf<T: BufRead>(b: T) -> Result<Srcinfo, Error> {
        Parser::parse(b)
    }

    /// Parse the file at a given path.
    ///
    /// ```
    /// # use srcinfo::Error;
    /// use srcinfo::Srcinfo;
    ///
    /// # fn test() -> Result<(), Error> {
    /// let file = ".SRCINFO";
    /// # let file = "tests/srcinfo/libc++";
    /// let srcinfo = Srcinfo::parse_file(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_file<P: AsRef<Path>>(s: P) -> Result<Srcinfo, Error> {
        let file = File::open(s)?;
        let buf = BufReader::new(file);
        Parser::parse(buf)
    }

    /// Builds a complete version string in the format: "epoch-pkgver-pkgrel".
    ///
    /// If the epoch is none then the epoch and connecting hyphen will be omitted.
    ///
    /// ```
    /// # use srcinfo::Error;
    /// use srcinfo::Srcinfo;
    ///
    /// # fn test() -> Result<(), Error> {
    /// let srcinfo: Srcinfo = "
    /// pkgbase = example
    /// pkgver = 1.5.0
    /// pkgrel = 5
    ///
    /// pkgname = example".parse()?;
    ///
    /// assert_eq!(srcinfo.version(), "1.5.0-5");
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> String {
        match self.epoch() {
            Some(epoch) => format!("{}:{}-{}", epoch, self.pkgver(), self.pkgrel()),
            None => format!("{}-{}", self.pkgver(), self.pkgrel()),
        }
    }

    /// Returns an Iterator over all the pkgnames the Package contains.
    ///
    /// ```
    /// # use srcinfo::Error;
    /// use srcinfo::Srcinfo;
    ///
    /// # fn test() -> Result<(), Error> {
    /// let srcinfo: Srcinfo = "
    /// pkgbase = example
    /// pkgver = 1.5.0
    /// pkgrel = 5
    /// pkgdesc = 1
    ///
    /// pkgname = example
    ///
    /// pkgname = foo
    /// pkgdesc = 2
    ///
    /// pkgname = bar
    /// pkgdesc = 3".parse()?;
    ///
    /// let mut names = srcinfo.names().collect::<Vec<_>>();
    /// assert_eq!(names, vec!["example", "foo", "bar"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.pkgs().iter().map(|p| p.pkgname.as_str())
    }

    /// Searches for a package with a given pkgname
    ///
    /// ```
    /// # use srcinfo::Error;
    /// use srcinfo::Srcinfo;
    ///
    /// # fn test() -> Result<(), Error> {
    /// let srcinfo: Srcinfo = "
    /// pkgbase = example
    /// pkgver = 1.5.0
    /// pkgrel = 5
    /// pkgdesc = 1
    ///
    /// pkgname = example
    ///
    /// pkgname = foo
    /// pkgdesc = 2
    ///
    /// pkgname = bar
    /// pkgdesc = 3".parse()?;
    ///
    /// let pkg = srcinfo.pkg("foo").unwrap();
    /// assert_eq!(pkg.pkgname, "foo");
    /// # Ok(())
    /// # }
    /// ```
    pub fn pkg<S: AsRef<str>>(&self, name: S) -> Option<&Package> {
        self.pkgs().iter().find(|p| p.pkgname == name.as_ref())
    }

    /// Returns the list of packages in this Srcinfo
    pub fn pkgs(&self) -> &[Package] {
        &self.pkgs
    }

    /// Returns the header comment
    pub fn comment(&self) -> &str {
        &self.comment
    }

    get!(pkgbase, base.pkgbase, &str);
    get!(pkgver, base.pkgver, &str);
    get!(pkgrel, base.pkgrel, &str);
    geto!(epoch, base.epoch, Option<&str>);
    get!(source, base.source, &[ArchVec]);
    get!(valid_pgp_keys, base.valid_pgp_keys, &[String]);
    get!(no_extract, base.no_extract, &[String]);
    get!(md5sums, base.md5sums, &[ArchVec]);
    get!(sha1sums, base.sha1sums, &[ArchVec]);
    get!(sha224sums, base.sha224sums, &[ArchVec]);
    get!(sha256sums, base.sha256sums, &[ArchVec]);
    get!(sha384sums, base.sha384sums, &[ArchVec]);
    get!(sha512sums, base.sha512sums, &[ArchVec]);
    get!(b2sums, base.b2sums, &[ArchVec]);
    get!(makedepends, base.makedepends, &[ArchVec]);
    get!(checkdepends, base.checkdepends, &[ArchVec]);

    geto!(pkgdesc, pkg.pkgdesc, Option<&str>);
    get!(arch, pkg.arch, &[String]);
    geto!(url, pkg.url, Option<&str>);
    get!(license, pkg.license, &[String]);
    get!(groups, pkg.groups, &[String]);
    get!(depends, pkg.depends, &[ArchVec]);
    get!(optdepends, pkg.optdepends, &[ArchVec]);
    get!(provides, pkg.provides, &[ArchVec]);
    get!(conflicts, pkg.conflicts, &[ArchVec]);
    get!(replaces, pkg.replaces, &[ArchVec]);
    get!(backup, pkg.backup, &[String]);
    get!(options, pkg.options, &[String]);
    geto!(install, pkg.install, Option<&str>);
    geto!(changelog, pkg.changelog, Option<&str>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use std::fs;

    #[test]
    fn test_supports() {
        let av = ArchVec::from("x86_64".to_string());
        assert!(av.supports("x86_64".to_string()));

        let av = ArchVec::default();
        assert!(av.supports("x86_64"));

        let av = ArchVec::from("i686");
        assert!(!av.supports("x86_64"));

        let srcinfo: Srcinfo = include_str!("../tests/srcinfo/libc++").parse().unwrap();
        let depends = srcinfo
            .base
            .makedepends
            .into_iter()
            .filter(|av| av.supports("x86_64"))
            .flat_map(|av| av.vec)
            .collect::<Vec<_>>();

        let expected = vec![
            "clang".to_string(),
            "cmake".to_string(),
            "ninja".to_string(),
            "python".to_string(),
            "libunwind".to_string(),
        ];

        assert_eq!(expected, depends);
    }

    #[test]
    fn test_parsable() {
        let path = fs::read_dir("tests/srcinfo/good").unwrap();

        for file in path.map(|x| x.unwrap()) {
            let srcinfo = Srcinfo::parse_file(file.path());
            assert!(srcinfo.is_ok(), "{:?} {:?}", file, srcinfo);
        }
    }

    fn assert_eq_libcpp(srcinfo: &Srcinfo) {
        let base = PackageBase {
                pkgbase: "libc++".to_string(),
                pkgver: "6.0.0".to_string(),
                pkgrel: "1".to_string(),
                source: vec![ArchVec {
                    arch: None,
                    vec: vec![
                        "https://releases.llvm.org/6.0.0/llvm-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/llvm-6.0.0.src.tar.xz.sig".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxx-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxx-6.0.0.src.tar.xz.sig".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxxabi-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxxabi-6.0.0.src.tar.xz.sig"
                            .to_string(),
                    ],
                }],
                valid_pgp_keys: vec![
                    "11E521D646982372EB577A1F8F0871F202119294".to_string(),
                    "B6C8F98282B944E3B0D5C2530FC3042E345AD05D".to_string(),
                ],
                no_extract: vec![
                    "llvm-6.0.0.src.tar.xz".to_string(),
                    "llvm-6.0.0.src.tar.xz.sig".to_string(),
                    "libcxx-6.0.0.src.tar.xz".to_string(),
                    "libcxx-6.0.0.src.tar.xz.sig".to_string(),
                    "libcxxabi-6.0.0.src.tar.xz".to_string(),
                    "libcxxabi-6.0.0.src.tar.xz.sig".to_string(),
                ],
                sha512sums: vec![ArchVec {
                    arch: None,
                    vec: vec![
                        "a71fdd5ddc46f01327ad891cfcc198febdbe10769c57f14d8a4fb7d514621ee4080e1a641200d3353c16a16731d390270499ec6cd3dc98fadc570f3eb6b52b8c".to_string(),
                        "SKIP".to_string(),
                        "3d93910f85a778f36c5f7a4429639008acba5713a2c8ac79a9de09463af6f9a388af45d39af23423a7223660701697ba067f3391f25d5a970973691dd88635e3".to_string(),
                        "SKIP".to_string(),
                        "c5e4cc05105770b42b20595fdbda5e1483be4582bc94335da1a15531ba43a0ecf30e1e0a252f62d4d0e6c79cda9d44ff5fdbe69a0a295b2431fd6de158410e2e".to_string(),
                        "SKIP".to_string(),
                    ]
                }],
                makedepends: vec![ArchVec {
                    arch: None,
                    vec: vec![
                        "clang".to_string(),
                        "cmake".to_string(),
                        "ninja".to_string(),
                        "python".to_string(),
                        "libunwind".to_string(),
                    ]
                }],
                ..Default::default()
            };

        let package = Package {
            arch: vec!["i686".to_string(), "x86_64".to_string()],
            url: Some("https://libcxx.llvm.org/".to_string()),
            license: vec![
                "MIT".to_string(),
                "custom:University of Illinois/NCSA Open Source License".to_string(),
            ],
            depends: vec![ArchVec {
                arch: None,
                vec: vec!["gcc-libs".to_string()],
            }],
            ..Default::default()
        };

        let expected = Srcinfo {
            comment: String::new(),
            base: base.clone(),
            pkg: package.clone(),
            pkgs: vec![
                Package {
                    pkgname: "libc++".to_string(),
                    pkgdesc: Some("LLVM C++ standard library.".to_string()),
                    depends: vec![ArchVec {
                        arch: None,
                        vec: vec!["libc++abi=6.0.0-1".to_string()],
                    }],
                    ..package.clone()
                },
                Package {
                    pkgname: "libc++abi".to_string(),
                    pkgdesc: Some(
                        "Low level support for the LLVM C++ standard library.".to_string(),
                    ),
                    ..package.clone()
                },
                Package {
                    pkgname: "libc++experimental".to_string(),
                    pkgdesc: Some("LLVM C++ experimental library.".to_string()),
                    depends: vec![ArchVec {
                        arch: None,
                        vec: vec!["libc++=6.0.0-1".to_string()],
                    }],
                    ..package.clone()
                },
            ],
        };

        assert_eq!(expected, *srcinfo);
        assert_eq!(srcinfo.version(), "6.0.0-1");
    }

    #[test]
    fn version() {
        let mut srcinfo = Srcinfo::default();

        srcinfo.base.pkgver = "1.2.3".to_string();
        srcinfo.base.pkgrel = "2".to_string();
        assert_eq!(srcinfo.version(), "1.2.3-2".to_string());

        srcinfo.base.epoch = Some("4".to_string());
        assert_eq!(srcinfo.version(), "4:1.2.3-2".to_string());
    }

    #[test]
    fn libcpp_str() {
        let srcinfo: Srcinfo = include_str!("../tests/srcinfo/libc++").parse().unwrap();
        assert_eq_libcpp(&srcinfo);
    }

    #[test]
    fn libcpp_buf() {
        let srcinfo =
            Srcinfo::parse_buf(include_str!("../tests/srcinfo/libc++").as_bytes()).unwrap();
        assert_eq_libcpp(&srcinfo);
    }

    #[test]
    fn libcpp_file() {
        let srcinfo = Srcinfo::parse_file("tests/srcinfo/libc++").unwrap();
        assert_eq_libcpp(&srcinfo);
    }

    #[test]
    fn gdc_bin() {
        let base = PackageBase {
            pkgbase: "gdc-bin".to_string(),
            pkgver: "6.3.0+2.068.2".to_string(),
            pkgrel: "1".to_string(),
            source: vec![
                ArchVec {
                    arch: Some("i686".to_string()),
                    vec: vec![
                        "http://gdcproject.org/downloads/binaries/6.3.0/i686-linux-gnu/gdc-6.3.0+2.068.2.tar.xz".to_string(),
                    ],
                },
                ArchVec {
                    arch: Some("x86_64".to_string()),
                    vec: vec![
                        "http://gdcproject.org/downloads/binaries/6.3.0/x86_64-linux-gnu/gdc-6.3.0+2.068.2.tar.xz".to_string(),
                    ]
                }
            ],
            md5sums: vec![
                ArchVec {
                    arch: Some("i686".to_string()),
                    vec: vec![
                        "cc8dcd66b189245e39296b1382d0dfcc".to_string(),
                    ],
                },
                ArchVec {
                    arch: Some("x86_64".to_string()),
                    vec: vec![
                        "16d3067ebb3938dba46429a4d9f6178f".to_string(),
                    ]
                }
            ],

            ..Default::default()
        };
        let package = Package {
            url: Some("https://gdcproject.org/".to_string()),
            arch: vec!["i686".to_string(), "x86_64".to_string()],
            license: vec!["GPL".to_string()],
            ..Default::default()
        };

        let expected = Srcinfo {
            comment: String::new(),
            base: base.clone(),
            pkg: package.clone(),
            pkgs: vec![
                Package {
                    pkgname: "gdc-bin".to_string(),
                    pkgdesc: Some("Compiler for D programming language which uses gcc backend".to_string()),
                    depends: vec![
                        ArchVec {
                            arch: None,
                            vec: vec![
                                "gdc-gcc".to_string(),
                                "perl".to_string(),
                                "binutils".to_string(),
                                "libgphobos".to_string(),
                            ]
                        }
                    ],
                    provides: vec![
                        ArchVec {
                            arch: None,
                            vec: vec![
                                "d-compiler=2.068.2".to_string(),
                                "gdc=6.3.0+2.068.2".to_string(),
                            ]
                        }
                    ],

                    ..package.clone()
                },
                Package {
                    pkgname: "gdc-gcc".to_string(),
                    pkgdesc: Some("The GNU Compiler Collection - C and C++ frontends (from GDC, gdcproject.org)".to_string()),
                    provides: vec![
                        ArchVec {
                            arch: None,
                            vec: vec![
                                "gcc=6.3.0".to_string(),
                                "gcc-libs=6.3.0".to_string(),
                            ]
                        }
                    ],
                    ..package.clone()
                },
                Package {
                    pkgname: "libgphobos-lib32".to_string(),
                    pkgdesc: Some("Standard library for D programming language, GDC port".to_string()),
                    provides: vec![
                        ArchVec {
                            arch: None,
                            vec: vec![
                                "d-runtime-lib32".to_string(),
                                "d-stdlib-lib32".to_string(),
                            ]
                        }
                    ],
                    ..package.clone()
                },

            ]
        };

        let srcinfo = include_str!("../tests/srcinfo/gdc-bin").parse().unwrap();
        assert_eq!(expected, srcinfo);
    }

    #[test]
    fn empty_override() {
        let expected = Srcinfo {
            comment: String::new(),
            base: PackageBase {
                pkgbase: "foo".to_string(),
                pkgver: "1".to_string(),
                pkgrel: "1".to_string(),
                ..Default::default()
            },
            pkg: Package {
                arch: vec!["foo".to_string()],
                ..Default::default()
            },
            pkgs: vec![Package {
                pkgname: "foo".to_string(),
                ..Default::default()
            }],
        };

        let srcinfo = include_str!("../tests/srcinfo/empty_override")
            .parse::<Srcinfo>()
            .unwrap();
        assert_eq!(expected, srcinfo);
    }

    #[test]
    fn error_undeclared_arch() {
        let err = include_str!("../tests/srcinfo/undeclared_arch")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 5);

        match err.kind {
            ErrorKind::UndeclaredArch(ref key, ref arch) => {
                assert_eq!(key, "depends_any");
                assert_eq!(arch, "any");
            }
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/undeclared_arch2")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 5);

        match err.kind {
            ErrorKind::UndeclaredArch(ref key, ref arch) => {
                assert_eq!(key, "depends_bar");
                assert_eq!(arch, "bar");
            }
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_no_pkgbase() {
        let err = include_str!("../tests/srcinfo/no_pkgbase")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 1);

        match err.kind {
            ErrorKind::KeyBeforePkgbase(ref key) => assert_eq!(key, "pkgdesc"),
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_duplicate_pkgbase() {
        let err = include_str!("../tests/srcinfo/duplicate_pkgbase")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 5);

        match err.kind {
            ErrorKind::DuplicatePkgbase => {}
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_key_after_pkgbase() {
        let err = include_str!("../tests/srcinfo/base_field_after_pkgname")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 7);

        match err.kind {
            ErrorKind::KeyAfterPkgname(ref key) => assert_eq!(key, "noextract"),
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_key_before_pkgbase() {
        let err = include_str!("../tests/srcinfo/pkgname_before_pkgbase")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 1);

        match err.kind {
            ErrorKind::KeyBeforePkgbase(ref key) => assert_eq!(key, "pkgname"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/key_before_pkgbase")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 1);

        match err.kind {
            ErrorKind::KeyBeforePkgbase(ref key) => assert_eq!(key, "arch"),
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_missing_field() {
        let res = include_str!("../tests/srcinfo/no_arch").parse::<Srcinfo>();
        assert!(res.is_ok());

        let err = include_str!("../tests/srcinfo/no_pkgrel")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgrel"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/no_pkgver")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgver"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/no_pkgname")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgname"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/empty")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgbase"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/no_pkgname")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgname"),
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_empty_key() {
        let err = include_str!("../tests/srcinfo/no_key")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 5);

        match err.kind {
            ErrorKind::EmptyKey => {}
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_empty_value() {
        let err = include_str!("../tests/srcinfo/no_value")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 3);

        match err.kind {
            ErrorKind::EmptyValue(ref key) => assert_eq!(key, "pkgver"),
            _ => panic!("{:?}", err),
        }

        let err = include_str!("../tests/srcinfo/no_value2")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 2);

        match err.kind {
            ErrorKind::EmptyValue(ref key) => assert_eq!(key, "arch"),
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn error_io_error() {
        let err = Srcinfo::parse_file("").unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::IoError(_) => {}
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn arch_in_package() {
        let srcinfo = include_str!("../tests/srcinfo/arch_in_package")
            .parse::<Srcinfo>()
            .unwrap();

        assert_eq!(srcinfo.pkgs[0].depends.len(), 2);
    }

    #[test]
    fn arch_in_package_override() {
        let err = include_str!("../tests/srcinfo/arch_in_package_override")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 10);

        match err.kind {
            ErrorKind::UndeclaredArch(ref key, ref arch) => {
                assert_eq!(key, "depends_x86_64");
                assert_eq!(arch, "x86_64");
            }
            _ => panic!("{:?}", err),
        }
    }

    #[test]
    fn comment_1() {
        let srcinfo = include_str!("../tests/srcinfo/comment-1")
            .parse::<Srcinfo>()
            .unwrap();

        assert_eq!(srcinfo.comment(), "123\n\nabc");
    }

    #[test]
    fn comment_2() {
        let srcinfo = include_str!("../tests/srcinfo/comment-2")
            .parse::<Srcinfo>()
            .unwrap();

        assert_eq!(srcinfo.comment(), "123\nabc");
    }

    #[test]
    fn arch_vecs() {
        let field = vec![
            ArchVec::with_arch("a", vec!["1".to_string()]),
            ArchVec::with_arch("b", vec!["2".to_string()]),
            ArchVec::with_arch("c", vec!["3".to_string()]),
            ArchVec::any(vec!["4".to_string()]),
        ];

        assert_eq!(
            ArchVec::supported(&field, "b",).next().unwrap(),
            "2".to_string()
        );
    }

    #[test]
    fn arch_vecs2() {
        let a = vec![
            ArchVec::with_arch("a", vec!["1".to_string()]),
            ArchVec::with_arch("b", vec!["2".to_string()]),
            ArchVec::with_arch("c", vec!["3".to_string()]),
            ArchVec::any(vec!["4".to_string()]),
        ];
        let b = vec![
            ArchVec::with_arch("a", vec!["1".to_string()]),
            ArchVec::with_arch("b", vec!["2".to_string()]),
            ArchVec::with_arch("c", vec!["3".to_string()]),
            ArchVec::any(vec!["4".to_string()]),
        ];
        let c = vec![
            ArchVec::with_arch("a", vec!["1".to_string()]),
            ArchVec::with_arch("b", vec!["2".to_string()]),
            ArchVec::with_arch("c", vec!["3".to_string()]),
            ArchVec::any(vec!["4".to_string()]),
        ];

        let vals = a
            .iter()
            .chain(&b)
            .chain(&c)
            .flat_map(|v| v.values("c"))
            .collect::<Vec<_>>();
        assert_eq!(
            vals,
            vec![
                &"3".to_string(),
                &"4".to_string(),
                &"3".to_string(),
                &"4".to_string(),
                &"3".to_string(),
                &"4".to_string(),
            ]
        );
    }
}
