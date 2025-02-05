use std::path::Path;

use generator::write_source_code;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    if cwd.ends_with("generator") {
        write_source_code(
            Path::new("Vulkan-Headers/registry/vk.xml"),
            "../ash-structure-utils/src",
        );
    } else {
        write_source_code(
            Path::new("generator/Vulkan-Headers/registry/vk.xml"),
            "ash-structure-utils/src",
        );
    }
}
