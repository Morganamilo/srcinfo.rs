use std::io::BufRead;

use crate::error::{Error, ErrorKind};
use crate::srcinfo::{ArchVec, Package, Srcinfo};

macro_rules! merge {
    ($slf:ident, $base:ident, $field:ident) => {
        if $base.$field.is_none() && !has_override(&$slf.empty_overrides, stringify!($field), None)
        {
            $base.$field = $slf.srcinfo.pkg.$field.clone();
        }
    };
}

macro_rules! merge_vec {
    ($slf:ident, $base:ident, $field:ident) => {
        if $base.$field.is_empty() && !has_override(&$slf.empty_overrides, stringify!($field), None)
        {
            $base.$field = $slf.srcinfo.pkg.$field.clone();
        }
    };
}

macro_rules! merge_arch_string {
    ($slf:ident, $base:ident, $field:ident) => {
        for arch_string in &$slf.srcinfo.pkg.$field {
            if !has_override(
                &$slf.empty_overrides,
                stringify!($field),
                arch_string.arch.as_deref(),
            ) {
                if !$base.$field.iter().any(|a| a.arch == arch_string.arch) {
                    $base.$field.push(arch_string.clone());
                }
            }
        }
    };
}

// Splits a "key = pair"
fn split_pair(s: &str) -> Result<(&str, Option<&str>), ErrorKind> {
    let split = s.split_once('=');
    let split = split.ok_or_else(|| ErrorKind::EmptyValue(s.to_string()))?;
    let (key, value) = (split.0.trim(), split.1.trim());
    if key.is_empty() {
        return Err(ErrorKind::EmptyKey);
    }
    Ok((key, empty_to_none(value)))
}

// splits depends_foo to ("depends", Some("foo"))
fn split_key_arch(s: &str) -> (&str, Option<&str>) {
    let mut split = s.splitn(2, '_');
    let key = split.next().unwrap();
    let arch = split.next();
    (key, arch)
}

fn empty_to_none(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

fn append_arch_strings(arch_strings: &mut Vec<ArchVec>, arch: Option<&str>, value: &str) {
    if let Some(vec) = arch_strings.iter_mut().find(|a| arch == a.arch.as_deref()) {
        vec.vec.push(value.to_string());
    } else {
        arch_strings.push(ArchVec::new(arch, vec![value.to_string()]));
    }
}

fn has_override(overrides: &[(String, Option<String>)], key: &str, arch: Option<&str>) -> bool {
    overrides
        .iter()
        .map(|(k, a)| (k.as_str(), a.as_deref()))
        .any(|x| x == (key, arch))
}

#[derive(Default)]
pub struct Parser {
    srcinfo: Srcinfo,
    empty_overrides: Vec<(String, Option<String>)>,
    has_pkg: bool,
}

impl Parser {
    pub fn parse<T: BufRead>(s: T) -> Result<Srcinfo, Error> {
        let mut parser = Parser::default();

        for (n, line) in s.lines().enumerate() {
            let line = line?;

            parser
                .parse_line(&line)
                .map_err(|e| Error::new(e, line.trim(), n + 1))?;
        }

        parser.merge_current_package();
        parser.check_missing()?;

        Ok(parser.srcinfo)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), ErrorKind> {
        let line = line.trim();

        if self.srcinfo.pkgbase().is_empty() && line.trim_start().starts_with('#') {
            let comment = line[1..].trim();
            if !self.srcinfo.comment.is_empty() {
                self.srcinfo.comment.push('\n');
            }
            self.srcinfo.comment.push_str(comment);
        }

        if line.is_empty() || line.starts_with('#') {
            return Ok(());
        }

        let (key, pair) = split_pair(line)?;
        self.set_header_or_field(key, pair)
    }

    fn add_override(&mut self, key: &str, arch: Option<&str>) {
        if !has_override(&self.empty_overrides, key, arch) {
            self.empty_overrides
                .push((key.to_string(), arch.map(|s| s.to_string())));
        }
    }

    fn last_pkg(&mut self) -> &mut Package {
        let pkg = &mut self.srcinfo.pkg;
        self.srcinfo.pkgs.last_mut().unwrap_or(pkg)
    }

    #[allow(clippy::cognitive_complexity)]
    fn merge_current_package(&mut self) {
        if let Some(package) = self.srcinfo.pkgs.last_mut() {
            merge!(self, package, pkgdesc);
            merge_vec!(self, package, arch);
            merge!(self, package, url);
            merge_vec!(self, package, license);
            merge_vec!(self, package, groups);
            merge_arch_string!(self, package, depends);
            merge_arch_string!(self, package, optdepends);
            merge_arch_string!(self, package, provides);
            merge_arch_string!(self, package, conflicts);
            merge_arch_string!(self, package, replaces);
            merge_vec!(self, package, backup);
            merge_vec!(self, package, options);
            merge!(self, package, install);
            merge!(self, package, changelog);
            self.empty_overrides.clear();
        }
    }

    fn check_missing(&self) -> Result<(), ErrorKind> {
        Err(ErrorKind::MissingField(
            if self.srcinfo.pkgbase().is_empty() {
                "pkgbase"
            } else if self.srcinfo.pkgs.is_empty() {
                "pkgname"
            } else if self.srcinfo.pkgver().is_empty() {
                "pkgver"
            } else if self.srcinfo.pkgrel().is_empty() {
                "pkgrel"
            } else {
                return Ok(());
            }
            .to_string(),
        ))
    }

    // check that the _arch prefix of a field actually exists and is not any
    fn check_arch(&self, arches: &[String], key: &str, arch: &str) -> Result<(), ErrorKind> {
        if arch == "any" || !arches.iter().any(|a| a.as_str() == arch) {
            Err(ErrorKind::UndeclaredArch(key.to_string(), arch.to_string()))
        } else {
            Ok(())
        }
    }

    fn check_not_arch_specific(&self, key: &str, arch: Option<&str>) -> Result<(), ErrorKind> {
        match arch {
            None => Ok(()),
            Some(_) => Err(ErrorKind::NotArchSpecific(key.to_string())),
        }
    }

    fn check_key_after_pkgname(&self, key: &str) -> Result<(), ErrorKind> {
        if self.has_pkg {
            Err(ErrorKind::KeyAfterPkgname(key.to_string()))
        } else {
            Ok(())
        }
    }

    fn push_pkg(&mut self, pkgname: &str) {
        let pkgname = pkgname.to_string();
        let pkg = Package {
            pkgname,
            ..Default::default()
        };

        self.merge_current_package();
        self.has_pkg = true;
        self.srcinfo.pkgs.push(pkg);
    }

    fn set_pkgbase(&mut self, value: Option<&str>) -> Result<(), ErrorKind> {
        if !self.srcinfo.pkgbase().is_empty() {
            return Err(ErrorKind::DuplicatePkgbase);
        }
        self.srcinfo.base.pkgbase = value
            .ok_or_else(|| ErrorKind::EmptyValue("pkgbase".to_string()))?
            .to_string();

        Ok(())
    }

    fn set_header_or_field(&mut self, key: &str, value: Option<&str>) -> Result<(), ErrorKind> {
        if key == "pkgbase" {
            self.set_pkgbase(value)
        } else if self.srcinfo.pkgbase().is_empty() {
            Err(ErrorKind::KeyBeforePkgbase(key.to_string()))
        } else if key == "pkgname" {
            let pkgname = value.ok_or_else(|| ErrorKind::EmptyValue(key.to_string()))?;
            self.push_pkg(pkgname);
            Ok(())
        } else {
            self.set_field(key, value)
        }
    }

    fn set_field(&mut self, key_arch: &str, value: Option<&str>) -> Result<(), ErrorKind> {
        let (key, arch) = split_key_arch(key_arch);

        let Some(value) = value else {
            if self.has_pkg {
                self.add_override(key, arch);
                return Ok(());
            } else {
                return Err(ErrorKind::EmptyValue(key.to_string()));
            }
        };

        if has_override(&self.empty_overrides, key, arch) {
            return Ok(());
        }

        if let Some(arch) = arch {
            let base = &self.srcinfo.pkg;
            let pkg = self.srcinfo.pkgs.last().unwrap_or(base);
            let pkg_arch =
                if pkg.arch.is_empty() && !has_override(&self.empty_overrides, "arch", None) {
                    &base.arch
                } else {
                    &pkg.arch
                };
            self.check_arch(pkg_arch, key_arch, arch)?;
        }

        if self.match_pkgbase(key, value) {
            self.check_not_arch_specific(key_arch, arch)?;
            self.check_key_after_pkgname(key_arch)?;
        } else if self.match_pkgbase_arch(key, arch, value) {
            self.check_key_after_pkgname(key_arch)?;
        } else if self.match_pkg(key, value) {
            self.check_not_arch_specific(key_arch, arch)?;
        } else {
            self.match_pkg_arch(key, arch, value);
        }

        Ok(())
    }

    fn match_pkgbase(&mut self, key: &str, value: &str) -> bool {
        let base = &mut self.srcinfo.base;
        match key {
            "pkgver" => base.pkgver = value.to_string(),
            "pkgrel" => base.pkgrel = value.to_string(),
            "epoch" => base.epoch = Some(value.to_string()),
            "validpgpkeys" => base.valid_pgp_keys.push(value.to_string()),
            "noextract" => base.no_extract.push(value.to_string()),
            _ => return false,
        }

        true
    }

    fn match_pkgbase_arch(&mut self, key: &str, arch: Option<&str>, value: &str) -> bool {
        let base = &mut self.srcinfo.base;
        match key {
            "source" => append_arch_strings(&mut base.source, arch, value),
            "md5sums" => append_arch_strings(&mut base.md5sums, arch, value),
            "sha1sums" => append_arch_strings(&mut base.sha1sums, arch, value),
            "sha224sums" => append_arch_strings(&mut base.sha224sums, arch, value),
            "sha256sums" => append_arch_strings(&mut base.sha256sums, arch, value),
            "sha384sums" => append_arch_strings(&mut base.sha384sums, arch, value),
            "sha512sums" => append_arch_strings(&mut base.sha512sums, arch, value),
            "b2sums" => append_arch_strings(&mut base.b2sums, arch, value),
            "makedepends" => append_arch_strings(&mut base.makedepends, arch, value),
            "checkdepends" => append_arch_strings(&mut base.checkdepends, arch, value),
            _ => return false,
        }

        true
    }

    fn match_pkg(&mut self, key: &str, value: &str) -> bool {
        let pkg = self.last_pkg();

        match key {
            "pkgdesc" => pkg.pkgdesc = Some(value.to_string()),
            "url" => pkg.url = Some(value.to_string()),
            "license" => pkg.license.push(value.to_string()),
            "install" => pkg.install = Some(value.to_string()),
            "changelog" => pkg.changelog = Some(value.to_string()),
            "groups" => pkg.groups.push(value.to_string()),
            "arch" => pkg.arch.push(value.to_string()),
            "backup" => pkg.backup.push(value.to_string()),
            "options" => pkg.options.push(value.to_string()),
            _ => return false,
        }

        true
    }

    fn match_pkg_arch(&mut self, key: &str, arch: Option<&str>, value: &str) -> bool {
        let pkg = self.last_pkg();

        match key {
            "depends" => append_arch_strings(&mut pkg.depends, arch, value),
            "optdepends" => append_arch_strings(&mut pkg.optdepends, arch, value),
            "conflicts" => append_arch_strings(&mut pkg.conflicts, arch, value),
            "provides" => append_arch_strings(&mut pkg.provides, arch, value),
            "replaces" => append_arch_strings(&mut pkg.replaces, arch, value),
            _ => return false,
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_pair() {
        assert_eq!(split_pair("a=b").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair("a==b").unwrap(), ("a", Some("=b")));
        assert_eq!(split_pair("a= b").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair("a =b").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair("a = b").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair(" a = b ").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair("\ta\t= b").unwrap(), ("a", Some("b")));
        assert_eq!(split_pair("a=").unwrap(), ("a", None));
        assert_eq!(split_pair(" a =").unwrap(), ("a", None));

        let err = split_pair("a").unwrap_err();
        match err {
            ErrorKind::EmptyValue(ref key) => assert_eq!(key, "a"),
            _ => panic!("{:?}", err),
        }

        assert!(split_pair("=b").is_err());
    }

    #[test]
    fn test_split_key_arch() {
        assert_eq!(split_key_arch("a_b"), ("a", Some("b")));
        assert_eq!(split_key_arch("a_b_c"), ("a", Some("b_c")));
        assert_eq!(split_key_arch("a"), ("a", None));
    }

    #[test]
    fn test_append_arch_strings() {
        let mut arch_strings = vec![ArchVec::from("x86_64")];
        append_arch_strings(&mut arch_strings, Some("arm"), "foo");

        assert_eq!(
            arch_strings,
            vec![
                ArchVec::from("x86_64"),
                ArchVec::new(Some("arm"), vec!["foo".to_string()]),
            ]
        );

        let mut arch_strings = vec![ArchVec::from("x86_64")];
        append_arch_strings(&mut arch_strings, Some("x86_64"), "foo");

        assert_eq!(
            arch_strings,
            vec![ArchVec::new(Some("x86_64"), vec!["foo".to_string()]),]
        );

        let mut arch_strings = vec![ArchVec::from("x86_64")];
        append_arch_strings(&mut arch_strings, Some("x86_64"), "foo");
        append_arch_strings(&mut arch_strings, Some("x86_64"), "bar");
        append_arch_strings(&mut arch_strings, Some("x86_64"), "a");
        append_arch_strings(&mut arch_strings, Some("x86_64"), "b");

        assert_eq!(
            arch_strings,
            vec![ArchVec::new(
                Some("x86_64"),
                vec![
                    "foo".to_string(),
                    "bar".to_string(),
                    "a".to_string(),
                    "b".to_string()
                ]
            ),]
        );

        let mut arch_strings = vec![ArchVec::from("x86_64")];
        append_arch_strings(&mut arch_strings, Some("x86_64"), "foo");
        append_arch_strings(&mut arch_strings, Some("arm"), "bar");
        append_arch_strings(&mut arch_strings, Some("x86_64"), "a");
        append_arch_strings(&mut arch_strings, Some("arm"), "b");

        assert_eq!(
            arch_strings,
            vec![
                ArchVec::new(Some("x86_64"), vec!["foo".to_string(), "a".to_string()]),
                ArchVec::new(Some("arm"), vec!["bar".to_string(), "b".to_string()]),
            ]
        );
    }
}
