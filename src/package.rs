use crate::symlink::{Symlink, SymlinkStatus};
use colored::Colorize;
use std::{
    fmt::{self, Display},
    fs::{self, DirEntry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub symlinks: Vec<Symlink>,
}

impl Package {
    pub fn new(dir: &DirEntry) -> Self {
        let mut symlinks = Vec::new();
        let symlinks_file = fs::read_to_string(dir.path().join(".symlink")).unwrap_or_default();

        for line in symlinks_file.lines() {
            let Some(mut symlink) = Symlink::from_str(dir.path(), line) else {
                continue;
            };

            let from = symlink.from_path();
            let to = symlink.to_path();

            if let Ok(path) = fs::read_link(to)
                && path == *from
            {
                symlink.status = SymlinkStatus::Linked;
            }

            symlinks.push(symlink);
        }

        Self {
            name: dir.file_name().display().to_string(),
            symlinks,
        }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let general_status = {
            let mut iter = self.symlinks.iter().map(|s| &s.status);
            match iter.next() {
                None => "(X) No symlinks defined".red().to_string(),
                Some(first) => {
                    if iter.all(|s| s == first) {
                        format!("{first}")
                    } else {
                        "(-) Mixed".yellow().to_string()
                    }
                }
            }
        };

        write!(
            f,
            "{} {} {}",
            self.name.bold(),
            "-".bold(),
            general_status.bold()
        )?;

        for symlink in &self.symlinks {
            write!(f, "{symlink}")?;
        }

        writeln!(f)
    }
}
