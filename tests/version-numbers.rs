#[test]
fn test_readme_deps_updated() {
    version_sync::assert_markdown_deps_updated!("README.md");
}

#[test]
fn test_readme_dependencies_badge_version() {
    let template = "[![dependency status](https://deps.rs/crate/identicon-server/{version}/status.svg)](https://deps.rs/crate/identicon-server/{version})";
    version_sync::assert_contains_substring!("README.md", template);
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/main.rs");
}
