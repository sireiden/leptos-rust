// ═══════════════════════════════════════════════════════════════════════════════
// 🌐 WASM ENTRY POINT - Bridge zwischen Browser JavaScript und Rust Code
// ═══════════════════════════════════════════════════════════════════════════════
// 
// Dieser File ist der "Startknopf" für unsere Rust-App im Browser.
// Ohne ihn würde der Browser unseren Rust-Code nicht finden können!

// 🔧 RUST ATTRIBUTE: #[wasm_bindgen::prelude::wasm_bindgen] 
// Das ist ein "Macro" (RUST Feature) das dem Compiler sagt:
// "Diese Funktion soll vom JavaScript im Browser aufrufbar sein"
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // 📦 RUST FEATURE: "use" Statement
    // Importiert alle öffentlichen Komponenten aus unserem "app" Crate
    // Das "::*" bedeutet "alles importieren" (wie "import * from app" in JavaScript)
    use app::*;
    
    // 🐛 LEPTOS FEATURE: Browser Logging Setup
    // Aktiviert Debug-Ausgaben in der Browser-Konsole (F12 → Console)
    // Das "_" vor dem "=" ist RUST Syntax für "ignoriere den Rückgabewert"
    _ = console_log::init_with_level(log::Level::Debug);
    
    // 💥 WASM FEATURE: Panic Handler
    // Wenn unser Rust-Code crashed, zeigt das lesbare Fehlermeldungen im Browser
    // Ohne das würden wir nur kryptische WASM-Fehler sehen
    console_error_panic_hook::set_once();

    // 🚀 LEPTOS FEATURE: App-Hydration  
    // Das ist der wichtigste Befehl! Er "erweckt" unsere App im Browser zum Leben
    // "hydrate_body" bedeutet: "Ersetze den <body> mit unserer reaktiven App"
    leptos::mount::hydrate_body(App);
}
