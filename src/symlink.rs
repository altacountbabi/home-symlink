use crate::expand_home;
use colored::Colorize;
use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    io::{self, ErrorKind},
    os::unix,
    path::PathBuf,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symlink {
    pub status: SymlinkStatus,
    pub base: PathBuf,
    pub kind: SymlinkKind,
}

impl Symlink {
    pub fn new(base: PathBuf, kind: SymlinkKind) -> Self {
        Self {
            status: SymlinkStatus::Unlinked,
            base,
            kind,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_path(&mut self) -> PathBuf {
        let path = match &self.kind {
            SymlinkKind::Whole { .. } => self.base.clone(),
            SymlinkKind::Map { from, .. } => self.base.join(from),
        };

        match path.canonicalize() {
            Ok(path) => path,
            Err(err) => {
                self.status = err.into();
                PathBuf::default()
            }
        }
    }

    pub fn to_path(&self) -> &PathBuf {
        match &self.kind {
            SymlinkKind::Whole { target } => target,
            SymlinkKind::Map { to, .. } => to,
        }
    }

    pub fn link(&mut self, force: bool) {
        let from = self.from_path();
        let to = self.to_path().clone();

        if force
            && let Err(_) = fs::remove_dir_all(&to)
            && let Err(err) = fs::remove_file(&to)
        {
            self.status = err.into();
        }

        match unix::fs::symlink(from, &to) {
            Ok(()) => self.status = SymlinkStatus::Linked,
            Err(err) => self.status = err.into(),
        }
    }

    pub fn unlink(&mut self, force: bool) {
        let from = self.from_path();
        let to = self.to_path().clone();

        if force {
            if let Err(err) = fs::remove_file(&to) {
                self.status = err.into();
            } else {
                self.status = SymlinkStatus::Unlinked;
            }
        }

        if let Ok(path) = fs::read_link(&to)
            && path == from
        {
            if let Err(err) = fs::remove_file(&to) {
                self.status = err.into();
            } else {
                self.status = SymlinkStatus::Unlinked;
            }
        } else {
            self.status = io::Error::new(
                ErrorKind::PermissionDenied,
                "Symlink wasn't created by home-symlink.",
            )
            .into();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymlinkStatus {
    Unlinked,
    Linked,
    Error { reason: String },
}

impl<E: Error> From<E> for SymlinkStatus {
    fn from(error: E) -> Self {
        Self::Error {
            reason: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymlinkKind {
    Whole { target: PathBuf },
    Map { from: PathBuf, to: PathBuf },
}

impl Symlink {
    pub fn from_str(base: PathBuf, s: &str) -> Option<Self> {
        let s = s.trim();

        let parts: Vec<&str> = s.splitn(2, '=').map(str::trim).collect();

        match parts.as_slice() {
            [single] => Some(Symlink::new(
                base,
                SymlinkKind::Whole {
                    target: expand_home(single),
                },
            )),
            [from, to] => Some(Symlink::new(
                base,
                SymlinkKind::Map {
                    from: expand_home(from),
                    to: expand_home(to),
                },
            )),
            _ => None,
        }
    }
}

impl Display for Symlink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SymlinkKind::Whole { target } => write!(
                f,
                "\n  {} -> {} - {}",
                self.base.display(),
                target.display(),
                self.status
            ),
            SymlinkKind::Map { from, to } => write!(
                f,
                "\n  {} -> {} - {}",
                self.base.join(from).display(),
                to.display(),
                self.status
            ),
        }
    }
}

impl Display for SymlinkStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymlinkStatus::Unlinked => write!(f, "{}", "(X) Unlinked".red()),
            SymlinkStatus::Error { reason } => {
                write!(f, "{}{reason}", "(X) Error: ".red())
            }
            SymlinkStatus::Linked => write!(f, "{}", "(✓) Linked".green()),
        }
    }
}
