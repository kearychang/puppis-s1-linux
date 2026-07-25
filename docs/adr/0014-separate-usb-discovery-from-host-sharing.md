# Separate USB discovery from host sharing

Inside the Rust application, USB network discovery and NetworkManager host sharing are separate deep modules joined by an opaque interface identifier and coordinated by the application module. Combining them would hide more correlation complexity, but the explicit split makes Linux USB/interface discovery and NetworkManager reconciliation independently inspectable and testable for learning; neither internal interface is exposed through Tauri.
