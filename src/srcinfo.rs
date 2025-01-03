use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::archvec::ArchVecs;
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

/// The fields from a .SRCINFO that only apply to the pkgbase.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct PackageBase {
    pub pkgbase: String,
    pub pkgver: String,
    pub pkgrel: String,
    pub epoch: Option<String>,
    pub source: ArchVecs,
    pub valid_pgp_keys: Vec<String>,
    pub no_extract: Vec<String>,
    pub md5sums: ArchVecs,
    pub sha1sums: ArchVecs,
    pub sha224sums: ArchVecs,
    pub sha256sums: ArchVecs,
    pub sha384sums: ArchVecs,
    pub sha512sums: ArchVecs,
    pub b2sums: ArchVecs,
    pub makedepends: ArchVecs,
    pub checkdepends: ArchVecs,
}

/// The fields from a .SRCINFO that are unique to each package.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Package {
    pub pkgname: String,
    pub pkgdesc: Option<String>,
    pub arch: Vec<String>,
    pub url: Option<String>,
    pub license: Vec<String>,
    pub groups: Vec<String>,
    pub depends: ArchVecs,
    pub optdepends: ArchVecs,
    pub provides: ArchVecs,
    pub conflicts: ArchVecs,
    pub replaces: ArchVecs,
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
    get!(depends, depends, &ArchVecs);
    get!(optdepends, optdepends, &ArchVecs);
    get!(provides, provides, &ArchVecs);
    get!(conflicts, conflicts, &ArchVecs);
    get!(replaces, replaces, &ArchVecs);
    get!(backup, backup, &[String]);
    get!(options, options, &[String]);
    geto!(install, install, Option<&str>);
    geto!(changelog, changelog, Option<&str>);
}

/// A complete representation of a .SRCINFO file.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
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
    /// // from_buf() is only used for the sake of example.
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
    /// Srcinfo::from_buf(buf)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_buf<T: BufRead>(b: T) -> Result<Srcinfo, Error> {
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
    /// let srcinfo = Srcinfo::from_path(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_path<P: AsRef<Path>>(s: P) -> Result<Srcinfo, Error> {
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
    /// let mut names = srcinfo.pkgnames().collect::<Vec<_>>();
    /// assert_eq!(names, vec!["example", "foo", "bar"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn pkgnames(&self) -> impl Iterator<Item = &str> {
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

    get!(comment, comment, &str);
    get!(pkgbase, base.pkgbase, &str);
    get!(pkgver, base.pkgver, &str);
    get!(pkgrel, base.pkgrel, &str);
    geto!(epoch, base.epoch, Option<&str>);
    get!(source, base.source, &ArchVecs);
    get!(valid_pgp_keys, base.valid_pgp_keys, &[String]);
    get!(no_extract, base.no_extract, &[String]);
    get!(md5sums, base.md5sums, &ArchVecs);
    get!(sha1sums, base.sha1sums, &ArchVecs);
    get!(sha224sums, base.sha224sums, &ArchVecs);
    get!(sha256sums, base.sha256sums, &ArchVecs);
    get!(sha384sums, base.sha384sums, &ArchVecs);
    get!(sha512sums, base.sha512sums, &ArchVecs);
    get!(b2sums, base.b2sums, &ArchVecs);
    get!(makedepends, base.makedepends, &ArchVecs);
    get!(checkdepends, base.checkdepends, &ArchVecs);

    geto!(pkgdesc, pkg.pkgdesc, Option<&str>);
    get!(arch, pkg.arch, &[String]);
    geto!(url, pkg.url, Option<&str>);
    get!(license, pkg.license, &[String]);
    get!(groups, pkg.groups, &[String]);
    get!(depends, pkg.depends, &ArchVecs);
    get!(optdepends, pkg.optdepends, &ArchVecs);
    get!(provides, pkg.provides, &ArchVecs);
    get!(conflicts, pkg.conflicts, &ArchVecs);
    get!(replaces, pkg.replaces, &ArchVecs);
    get!(backup, pkg.backup, &[String]);
    get!(options, pkg.options, &[String]);
    geto!(install, pkg.install, Option<&str>);
    geto!(changelog, pkg.changelog, Option<&str>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArchVec, ErrorKind};
    use std::fs;

    #[test]
    fn test_parsable() {
        let path = fs::read_dir("tests/srcinfo/good").unwrap();

        for file in path.map(|x| x.unwrap()) {
            let srcinfo = Srcinfo::from_path(file.path());
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
                    values: vec![
                        "https://releases.llvm.org/6.0.0/llvm-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/llvm-6.0.0.src.tar.xz.sig".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxx-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxx-6.0.0.src.tar.xz.sig".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxxabi-6.0.0.src.tar.xz".to_string(),
                        "https://releases.llvm.org/6.0.0/libcxxabi-6.0.0.src.tar.xz.sig"
                            .to_string(),
                    ],
                }].into(),
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
                    values: vec![
                        "a71fdd5ddc46f01327ad891cfcc198febdbe10769c57f14d8a4fb7d514621ee4080e1a641200d3353c16a16731d390270499ec6cd3dc98fadc570f3eb6b52b8c".to_string(),
                        "SKIP".to_string(),
                        "3d93910f85a778f36c5f7a4429639008acba5713a2c8ac79a9de09463af6f9a388af45d39af23423a7223660701697ba067f3391f25d5a970973691dd88635e3".to_string(),
                        "SKIP".to_string(),
                        "c5e4cc05105770b42b20595fdbda5e1483be4582bc94335da1a15531ba43a0ecf30e1e0a252f62d4d0e6c79cda9d44ff5fdbe69a0a295b2431fd6de158410e2e".to_string(),
                        "SKIP".to_string(),
                    ]
                }].into(),
                makedepends: vec![ArchVec {
                    arch: None,
                    values: vec![
                        "clang".to_string(),
                        "cmake".to_string(),
                        "ninja".to_string(),
                        "python".to_string(),
                        "libunwind".to_string(),
                    ]
                }].into(),
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
                values: vec!["gcc-libs".to_string()],
            }]
            .into(),
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
                        values: vec!["libc++abi=6.0.0-1".to_string()],
                    }]
                    .into(),
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
                        values: vec!["libc++=6.0.0-1".to_string()],
                    }]
                    .into(),
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
            Srcinfo::from_buf(include_str!("../tests/srcinfo/libc++").as_bytes()).unwrap();
        assert_eq_libcpp(&srcinfo);
    }

    #[test]
    fn libcpp_file() {
        let srcinfo = Srcinfo::from_path("tests/srcinfo/libc++").unwrap();
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
                    values: vec![
                        "http://gdcproject.org/downloads/binaries/6.3.0/i686-linux-gnu/gdc-6.3.0+2.068.2.tar.xz".to_string(),
                    ],
                },
                ArchVec {
                    arch: Some("x86_64".to_string()),
                    values: vec![
                        "http://gdcproject.org/downloads/binaries/6.3.0/x86_64-linux-gnu/gdc-6.3.0+2.068.2.tar.xz".to_string(),
                    ]
                }
            ].into(),
            md5sums: vec![
                ArchVec {
                    arch: Some("i686".to_string()),
                    values: vec![
                        "cc8dcd66b189245e39296b1382d0dfcc".to_string(),
                    ],
                },
                ArchVec {
                    arch: Some("x86_64".to_string()),
                    values: vec![
                        "16d3067ebb3938dba46429a4d9f6178f".to_string(),
                    ]
                }
            ].into(),

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
                            values: vec![
                                "gdc-gcc".to_string(),
                                "perl".to_string(),
                                "binutils".to_string(),
                                "libgphobos".to_string(),
                            ]
                        }
                    ].into(),
                    provides: vec![
                        ArchVec {
                            arch: None,
                            values: vec![
                                "d-compiler=2.068.2".to_string(),
                                "gdc=6.3.0+2.068.2".to_string(),
                            ]
                        }
                    ].into(),

                    ..package.clone()
                },
                Package {
                    pkgname: "gdc-gcc".to_string(),
                    pkgdesc: Some("The GNU Compiler Collection - C and C++ frontends (from GDC, gdcproject.org)".to_string()),
                    provides: vec![
                        ArchVec {
                            arch: None,
                            values: vec![
                                "gcc=6.3.0".to_string(),
                                "gcc-libs=6.3.0".to_string(),
                            ]
                        }
                    ].into(),
                    ..package.clone()
                },
                Package {
                    pkgname: "libgphobos-lib32".to_string(),
                    pkgdesc: Some("Standard library for D programming language, GDC port".to_string()),
                    provides: vec![
                        ArchVec {
                            arch: None,
                            values: vec![
                                "d-runtime-lib32".to_string(),
                                "d-stdlib-lib32".to_string(),
                            ]
                        }
                    ].into(),
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
        let err = Srcinfo::from_path("").unwrap_err();
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
}
