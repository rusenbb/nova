//! Extension scaffolding for `nova create extension`.

use anyhow::{bail, Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use std::fs;
use std::path::Path;

/// Create a new extension with the given name.
///
/// If title, description, or author are None, will prompt interactively (if terminal available)
/// or use defaults (if no terminal).
pub fn create_extension(
    name: &str,
    title: Option<String>,
    description: Option<String>,
    author: Option<String>,
) -> Result<()> {
    let dir = Path::new(name);

    // Validate name
    if !is_valid_extension_name(name) {
        bail!(
            "Invalid extension name '{}'. Use lowercase letters, numbers, and hyphens only.",
            name
        );
    }

    // Check if directory already exists
    if dir.exists() {
        bail!("Directory '{}' already exists", name);
    }

    println!(
        "{} {}",
        style("Creating extension in").cyan(),
        style(format!("./{}/", name)).cyan().bold()
    );
    println!();

    // Use provided values or prompt/default
    let is_interactive = console::Term::stdout().is_term();
    let theme = ColorfulTheme::default();

    let title: String = if let Some(t) = title {
        t
    } else if is_interactive {
        Input::with_theme(&theme)
            .with_prompt("Extension title")
            .default(to_title_case(name))
            .interact_text()?
    } else {
        to_title_case(name)
    };

    let description: String = if let Some(d) = description {
        d
    } else if is_interactive {
        Input::with_theme(&theme)
            .with_prompt("Description")
            .default(format!("A Nova extension for {}", title.to_lowercase()))
            .interact_text()?
    } else {
        format!("A Nova extension for {}", title.to_lowercase())
    };

    let author: String = if let Some(a) = author {
        a
    } else if is_interactive {
        Input::with_theme(&theme)
            .with_prompt("Author")
            .default(get_git_user().unwrap_or_else(|| "Your Name".to_string()))
            .interact_text()?
    } else {
        get_git_user().unwrap_or_else(|| "Your Name".to_string())
    };

    println!();

    // Create directory structure
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("assets"))?;

    // Generate files
    let manifest = generate_manifest(name, &title, &description, &author);
    let index_tsx = generate_index_tsx(&title);
    let tsconfig = generate_tsconfig();
    let package_json = generate_package_json(name);

    fs::write(dir.join("nova.toml"), manifest).context("Failed to write nova.toml")?;
    fs::write(dir.join("src/index.tsx"), index_tsx).context("Failed to write src/index.tsx")?;
    fs::write(dir.join("tsconfig.json"), tsconfig).context("Failed to write tsconfig.json")?;
    fs::write(dir.join("package.json"), package_json).context("Failed to write package.json")?;

    // Create placeholder icon
    create_placeholder_icon(dir)?;

    // Print success message
    println!("{}", style("Created:").green().bold());
    println!("  {}/", name);
    println!("  ├── nova.toml");
    println!("  ├── src/");
    println!("  │   └── index.tsx");
    println!("  ├── assets/");
    println!("  │   └── icon.png");
    println!("  ├── tsconfig.json");
    println!("  └── package.json");
    println!();
    println!("{}", style("Next steps:").cyan().bold());
    println!("  cd {}", name);
    println!("  npm install");
    println!("  nova dev");

    Ok(())
}

fn is_valid_extension_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

fn to_title_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_git_user() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
}

fn generate_manifest(name: &str, title: &str, description: &str, author: &str) -> String {
    format!(
        r#"[extension]
name = "{name}"
title = "{title}"
description = "{description}"
version = "0.1.0"
author = "{author}"
icon = "assets/icon.png"

[[commands]]
name = "index"
title = "{title}"
description = "{description}"
keywords = []
mode = "list"

[permissions]
# clipboard = true
# network = ["api.example.com"]
"#
    )
}

fn generate_index_tsx(_title: &str) -> String {
    r#"import {
  List,
  Icon,
  registerCommand,
  render,
} from "@aspect/nova";

function MainView() {
  return (
    <List searchBarPlaceholder="Search...">
      <List.Item
        id="hello"
        title="Hello, Nova!"
        subtitle="Welcome to your new extension"
        icon={Icon.system("star.fill")}
      />
      <List.Item
        id="docs"
        title="Read the Docs"
        subtitle="Learn how to build extensions"
        icon={Icon.system("book")}
      />
    </List>
  );
}

registerCommand("index", () => {
  render(() => <MainView />);
});
"#
    .to_string()
}

fn generate_tsconfig() -> String {
    r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "jsxImportSource": "@aspect/nova",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": ["src/**/*"]
}
"#
    .to_string()
}

fn generate_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "build": "esbuild src/index.tsx --bundle --outfile=dist/index.js --format=esm --external:@aspect/nova",
    "dev": "esbuild src/index.tsx --bundle --outfile=dist/index.js --format=esm --external:@aspect/nova --watch"
  }},
  "dependencies": {{
    "@aspect/nova": "^0.1.0"
  }},
  "devDependencies": {{
    "esbuild": "^0.20.0",
    "typescript": "^5.3.0"
  }}
}}
"#
    )
}

fn create_placeholder_icon(dir: &Path) -> Result<()> {
    // Create a minimal 1x1 transparent PNG as placeholder
    // This is a valid PNG file (smallest possible)
    #[rustfmt::skip]
    let png_data: [u8; 65] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature (8)
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk length + type (8)
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 (8)
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // bit depth, color type, CRC (9)
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk (8)
        0x08, 0xD7, 0x63, 0xF8, 0x0F, 0x00, 0x00, 0x01, 0x01, 0x00, 0x05, 0x1B, // data (12)
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND (12)
    ];

    fs::write(dir.join("assets/icon.png"), png_data).context("Failed to write icon.png")?;
    Ok(())
}
