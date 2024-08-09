use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn print_directory_tree<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", path.display())?;
    print_directory_tree_recursive(path, "", &mut handle)?;
    Ok(())
}

pub(crate) fn print_directory_tree_recursive<P: AsRef<Path>>(
    path: P,
    prefix: &str,
    handle: &mut impl Write,
) -> io::Result<()> {
    let path = path.as_ref();
    let entries = fs::read_dir(path)?;

    let mut entries: Vec<_> = entries.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        let is_last = i == entries.len() - 1;
        let new_prefix = if is_last { "└── " } else { "├── " };
        let continuation_prefix = if is_last { "    " } else { "│   " };

        writeln!(
            handle,
            "{}{}{}",
            prefix,
            new_prefix,
            entry.file_name().to_string_lossy()
        )?;

        if metadata.is_dir() {
            print_directory_tree_recursive(
                &path,
                &format!("{}{}", prefix, continuation_prefix),
                handle,
            )?;
        }
    }
    Ok(())
}
