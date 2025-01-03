use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{ArchVec, Package, Srcinfo};

fn write_val_arch(w: &mut Formatter<'_>, key: &str, arch: Option<&str>, value: &str) -> FmtResult {
    match arch {
        Some(arch) => write!(w, "\n\t{}_{} = {}", key, arch, value),
        None => write!(w, "\n\t{} = {}", key, value),
    }
}

fn write_val(w: &mut Formatter<'_>, key: &str, value: &str) -> FmtResult {
    write_val_arch(w, key, None, value)
}

fn write_arch_vec(w: &mut Formatter<'_>, key: &str, values: &ArchVec) -> FmtResult {
    for value in values.all() {
        write_val_arch(w, key, values.arch(), value)?;
    }
    Ok(())
}

fn write_arch_vecs(w: &mut Formatter<'_>, key: &str, values: &[ArchVec]) -> FmtResult {
    for vec in values {
        write_arch_vec(w, key, vec)?;
    }
    Ok(())
}

fn write_arr<S: AsRef<str>>(
    w: &mut Formatter<'_>,
    key: &str,
    values: impl IntoIterator<Item = S>,
) -> FmtResult {
    for value in values {
        write_val(w, key, value.as_ref())?;
    }
    Ok(())
}

fn write_pkg_val(w: &mut Formatter<'_>, k: &str, v: Option<&str>, base: Option<&str>) -> FmtResult {
    if v != base {
        write_val(w, k, v.unwrap_or_default())?;
    }
    Ok(())
}

fn write_pkg_arr(w: &mut Formatter<'_>, k: &str, v: &[String], base: &[String]) -> FmtResult {
    match (v != base, v.is_empty()) {
        (true, true) => write_val(w, k, ""),
        (true, false) => write_arr(w, k, v),
        _ => Ok(()),
    }
}

fn write_pkg_arch_vecs(
    w: &mut Formatter<'_>,
    key: &str,
    values: &[ArchVec],
    base: &[ArchVec],
) -> FmtResult {
    for value in values {
        match base.iter().find(|v| value.arch() == v.arch()) {
            Some(base) if base != value => write_arch_vec(w, key, value)?,
            None if !value.vec.is_empty() => write_arch_vec(w, key, value)?,
            _ => (),
        }
    }

    for base in base {
        if !base.vec.is_empty() && !values.iter().any(|v| base.arch() == v.arch()) {
            write_val_arch(w, key, base.arch(), "")?;
        }
    }

    Ok(())
}

impl Display for Srcinfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write_all(f)
    }
}

impl Srcinfo {
    fn write_comment(&self, w: &mut Formatter<'_>) -> FmtResult {
        for comment in self.comment().lines() {
            writeln!(w, "# {}", comment)?;
        }
        Ok(())
    }

    fn write_pkg(&self, pkg: &Package, w: &mut Formatter<'_>) -> FmtResult {
        write!(w, "\n\npkgname = {}", pkg.pkgname())?;
        write_pkg_val(w, "pkgdesc", pkg.pkgdesc(), self.pkgdesc())?;
        write_pkg_val(w, "url", pkg.url(), self.url())?;
        write_pkg_val(w, "install", pkg.install(), self.install())?;
        write_pkg_val(w, "changelog", pkg.changelog(), self.changelog())?;
        write_pkg_arr(w, "arch", pkg.arch(), self.arch())?;
        write_pkg_arr(w, "groups", pkg.groups(), self.groups())?;
        write_pkg_arr(w, "license", pkg.license(), self.license())?;
        write_pkg_arch_vecs(w, "depends", pkg.depends(), self.depends())?;
        write_pkg_arch_vecs(w, "optdepends", pkg.optdepends(), self.optdepends())?;
        write_pkg_arch_vecs(w, "provides", pkg.provides(), self.provides())?;
        write_pkg_arch_vecs(w, "conflicts", pkg.conflicts(), self.conflicts())?;
        write_pkg_arch_vecs(w, "replaces", pkg.replaces(), self.replaces())?;
        write_pkg_arr(w, "options", pkg.options(), self.options())?;
        write_pkg_arr(w, "backup", pkg.backup(), self.backup())?;
        Ok(())
    }

    fn write_all(&self, w: &mut Formatter<'_>) -> FmtResult {
        self.write_comment(w)?;
        write!(w, "pkgbase = {}", self.pkgbase())?;
        write_arr(w, "pkgdesc", self.pkgdesc())?;
        write_val(w, "pkgver", self.pkgver())?;
        write_val(w, "pkgrel", self.pkgrel())?;
        write_arr(w, "epoch", self.epoch())?;
        write_arr(w, "url", self.url())?;
        write_arr(w, "install", self.install())?;
        write_arr(w, "changelog", self.changelog())?;
        write_arr(w, "arch", self.arch())?;
        write_arr(w, "groups", self.groups())?;
        write_arr(w, "license", self.license())?;
        write_arch_vecs(w, "checkdepends", self.checkdepends())?;
        write_arch_vecs(w, "makedepends", self.makedepends())?;
        write_arch_vecs(w, "depends", self.depends())?;
        write_arch_vecs(w, "optdepends", self.optdepends())?;
        write_arch_vecs(w, "provides", self.provides())?;
        write_arch_vecs(w, "conflicts", self.conflicts())?;
        write_arch_vecs(w, "replaces", self.replaces())?;
        write_arr(w, "noextract", self.no_extract())?;
        write_arr(w, "options", self.options())?;
        write_arr(w, "backup", self.backup())?;
        write_arch_vecs(w, "source", self.source())?;
        write_arr(w, "validpgpkeys", self.valid_pgp_keys())?;
        write_arch_vecs(w, "md5sums", self.md5sums())?;
        write_arch_vecs(w, "sha1sums", self.sha1sums())?;
        write_arch_vecs(w, "sha224sums", self.sha224sums())?;
        write_arch_vecs(w, "sha256sums", self.sha256sums())?;
        write_arch_vecs(w, "sha384sums", self.sha384sums())?;
        write_arch_vecs(w, "sha512sums", self.sha512sums())?;
        write_arch_vecs(w, "b2sums", self.b2sums())?;

        for pkg in &self.pkgs {
            self.write_pkg(pkg, w)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{read_dir, read_to_string};

    use super::*;

    #[test]
    fn test_fmt() {
        for file in read_dir("tests/srcinfo/good/").unwrap() {
            let file = file.unwrap();

            let original = read_to_string(file.path()).unwrap();
            let srcinfo = original.parse::<Srcinfo>().unwrap();
            let srcinfo = srcinfo.to_string();

            let mut original = original.lines().collect::<Vec<_>>();
            let mut srcinfo = srcinfo.lines().collect::<Vec<_>>();

            original.sort();
            srcinfo.sort();
            assert_eq!(original, srcinfo);
        }
    }
}
