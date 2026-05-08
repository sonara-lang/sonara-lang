use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn preprocess(source: &str, base_dir: &Path, visited: &mut HashSet<String>) -> Result<String, String> {
    let mut result = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(raw) = trimmed.strip_prefix("import ") {
            let name = raw.trim().trim_matches('"');
            let import_path = base_dir.join(format!("{}.son", name));
            let canonical = import_path
                .canonicalize()
                .unwrap_or_else(|_| import_path.clone())
                .to_string_lossy()
                .to_string();

            if visited.contains(&canonical) {
                return Err(format!("circular import detected: {}", name));
            }

            let import_source = fs::read_to_string(&import_path)
                .map_err(|_| format!("import not found: '{}.son' (looked in {})", name, base_dir.display()))?;

            visited.insert(canonical);
            let expanded = preprocess(&import_source, base_dir, visited)?;
            result.push_str(&expanded);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    Ok(result)
}
