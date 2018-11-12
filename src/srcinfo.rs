use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::error::Error;
use crate::parse::Parser;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArchVec {
    pub arch: Option<String>,
    pub vec: Vec<String>,
}

impl ArchVec {
    #[cfg(test)]
    pub(crate) fn from_arch<S: Into<String>>(arch: S) -> ArchVec {
        ArchVec {
            arch: Some(arch.into()),
            vec: Vec::new(),
        }
    }

    pub(crate) fn from_vec<S: Into<String>>(arch: Option<S>, values: Vec<String>) -> ArchVec {
        ArchVec {
            arch: arch.map(|x| x.into()),
            vec: values,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    pub makedepends: Vec<ArchVec>,
    pub checkdepends: Vec<ArchVec>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Srcinfo {
    pub base: PackageBase,
    pub package: Package,
    pub packages: Vec<Package>,
}

impl FromStr for Srcinfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::parse(s.as_bytes())
    }
}

impl Srcinfo {
    pub fn parse_buf<T: BufRead>(b: T) -> Result<Srcinfo, Error> {
        Parser::parse(b)
    }

    pub fn parse_file<P: AsRef<Path>>(s: P) -> Result<Srcinfo, Error> {
        let file = File::open(s)?;
        let buf = BufReader::new(file);
        Parser::parse(buf)
    }

    pub fn version(&self) -> String {
        let base = &self.base;
        if let Some(ref epoch) = base.epoch {
            format!("{}:{}-{}", epoch, base.pkgver, base.pkgrel)
        } else {
            format!("{}-{}", base.pkgver, base.pkgrel)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use std::fs;

    #[test]
    fn test_parsable() {
        let path = fs::read_dir("tests/srcinfo/good").unwrap();

        for file in path.map(|x| x.unwrap()) {
            let srcinfo = Srcinfo::parse_file(file.path());
            assert!(srcinfo.is_ok(), format!("{:?} {:?}", file, srcinfo));
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
            base: base.clone(),
            package: package.clone(),
            packages: vec![
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
            base: base.clone(),
            package: package.clone(),
            packages: vec![
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
            base: PackageBase {
                pkgbase: "foo".to_string(),
                pkgver: "1".to_string(),
                pkgrel: "1".to_string(),
                ..Default::default()
            },
            package: Package {
                arch: vec!["foo".to_string()],
                ..Default::default()
            },
            packages: vec![Package {
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/key_before_pkgbase")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 1);

        match err.kind {
            ErrorKind::KeyBeforePkgbase(ref key) => assert_eq!(key, "arch"),
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/no_pkgver")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgver"),
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/no_pkgname")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgname"),
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/empty")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgbase"),
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/no_pkgname")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::MissingField(ref key) => assert_eq!(key, "pkgname"),
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
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
            _ => panic!(format!("{:?}", err)),
        }

        let err = include_str!("../tests/srcinfo/no_value2")
            .parse::<Srcinfo>()
            .unwrap_err();
        assert_eq!(err.line.as_ref().unwrap().number, 2);

        match err.kind {
            ErrorKind::EmptyValue(ref key) => assert_eq!(key, "arch"),
            _ => panic!(format!("{:?}", err)),
        }
    }

    #[test]
    fn error_io_error() {
        let err = Srcinfo::parse_file("").unwrap_err();
        assert_eq!(err.line, None);

        match err.kind {
            ErrorKind::IoError(_) => {}
            _ => panic!(format!("{:?}", err)),
        }
    }
}
