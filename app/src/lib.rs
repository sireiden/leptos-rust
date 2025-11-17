// ═══════════════════════════════════════════════════════════════════════════════
// 📦 IMPORTS - Alle externen Libraries die wir brauchen
// ═══════════════════════════════════════════════════════════════════════════════

// 🚀 LEPTOS FEATURE: Core Framework
// Das "prelude::*" importiert alle wichtigsten Leptos-Funktionen
// Ähnlich wie "import React from 'react'" in JavaScript
use leptos::prelude::*;

// 🏷️ LEPTOS FEATURE: Meta Tags & SEO 
// Tools um <title>, <meta> Tags etc. zu verwalten (wichtig für SEO)
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};

// 🧭 LEPTOS FEATURE: Client-Side Routing
// Ermöglicht mehrere "Seiten" ohne Server-Requests (wie React Router)
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

// 🔧 RUST FEATURE: Conditional Compilation
// #[cfg(feature = "hydrate")] bedeutet: "Nur kompilieren wenn im Browser"
// Diese Imports funktionieren NUR im Browser, nicht auf dem Server

// 📡 RUST FEATURE: JSON Handling (nur im Browser)
#[cfg(feature = "hydrate")]
use serde::Deserialize;

// 🌐 WASM FEATURE: JavaScript Interop (nur im Browser) 
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════════
// 🏠 HTML SHELL - Das "Grundgerüst" der Webseite
// ═══════════════════════════════════════════════════════════════════════════════
// 
// Diese Funktion erstellt das HTML-Grundgerüst. Läuft nur auf dem SERVER!
// Vergleichbar mit index.html in normalen Web-Apps

// 🔧 RUST FEATURE: Function Definition
// "pub" = öffentlich, "fn" = function, "-> impl IntoView" = Rückgabe-Typ
pub fn shell(options: LeptosOptions) -> impl IntoView {
    
    // 🎨 LEPTOS FEATURE: view! Macro  
    // Das ist Leptos' "JSX" - HTML direkt in Rust schreiben!
    // Wird zur Compile-Zeit in echten Rust-Code umgewandelt
    view! {
        // 📄 Standard HTML5 Deklaration
        <!DOCTYPE html>
        <html lang="en">
            <head>
                // 🌐 UTF-8 Encoding für internationale Zeichen
                <meta charset="utf-8"/>
                // 📱 Responsive Design für Handy-Bildschirme  
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                
                // 🔄 LEPTOS FEATURE: Auto-Reload (nur in Development)
                // Lädt die Seite automatisch neu wenn du Code änderst
                <AutoReload options=options.clone()/>
                
                // ⚡ LEPTOS FEATURE: Hydration Scripts
                // JavaScript Code der die statische HTML zu reaktiver App macht
                <HydrationScripts options/>
                
                // 🏷️ LEPTOS FEATURE: Meta Tags Container
                // Hier können Components <title> und <meta> Tags einfügen
                <MetaTags/>
            </head>
            <body>
                // 🚀 UNSERE HAUPT-APP! 
                // Das <App/> wird durch unsere reaktive Leptos-App ersetzt
                <App/>
            </body>
        </html>
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🚀 HAUPT-COMPONENT - Unsere komplette App!
// ═══════════════════════════════════════════════════════════════════════════════

// 🧩 LEPTOS FEATURE: Component Definition  
// #[component] ist ein Leptos-Macro das diese Funktion zur React-ähnlichen Component macht
#[component]
pub fn App() -> impl IntoView {
    
    // 🏷️ LEPTOS FEATURE: Meta Context
    // Ermöglicht Components das Setzen von <title> und <meta> tags
    provide_meta_context();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 📊 REACTIVE STATE - Das "Gehirn" unserer App
    // ═══════════════════════════════════════════════════════════════════════════
    // 
    // LEPTOS FEATURE: RwSignal = "Read-Write Signal" 
    // Das ist wie "useState" in React - automatische UI-Updates bei Änderungen!
    
    // 💰 Marktpreise: Symbol → Liste der letzten Preise
    // RUST TYPE: HashMap<String, Vec<f64>> = Map von Text zu Zahlen-Liste
    let prices = RwSignal::new(std::collections::HashMap::<String, Vec<f64>>::new());
    
    // 📈 Trade-History: Liste der letzten 100 Trades  
    // RUST TYPE: Vec<(String, f64, String)> = Liste von (Symbol, Preis, Seite)
    let trades = RwSignal::new(Vec::<(String, f64, String)>::new());
    
    // 📖 Orderbook-Daten: Symbol → (Bid-Preise, Ask-Preise)
    // RUST TYPE: HashMap<String, (Vec<f64>, Vec<f64>)> = Map zu Tupel von Listen
    let book_depth = RwSignal::new(std::collections::HashMap::<String, (Vec<f64>, Vec<f64>)>::new());
    
    // 📊 Performance Metriken für Live-Monitoring:
    let msg_rate = RwSignal::new(Vec::<f64>::new());        // Momentane Messages pro Sekunde (CLIENT-SEITIG!)
    let latency_values = RwSignal::new(Vec::<f64>::new());  // Latenz in Millisekunden  
    let fps_values = RwSignal::new(Vec::<f64>::new());      // Frames pro Sekunde
    let sample_max = RwSignal::new(200usize);               // Maximale Chart-Datenpunkte
    let msg_count = RwSignal::new(0u64);                    // Message Counter für aktuelle Sekunde
    let msg_rate_timer = RwSignal::new(0.0);                // Timer für Rate-Berechnung
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 🌐 WEBSOCKET CONNECTION - Live-Daten vom Server  
    // ═══════════════════════════════════════════════════════════════════════════
    
    // 🔄 LEPTOS FEATURE: Effect (Side Effect Management)
    // Läuft automatisch wenn die App startet - wie useEffect() in React  
    // "move |_|" ist RUST Syntax für "Closure die Ownership übernimmt"
    Effect::new(move |_| {
        
        // 🔧 RUST FEATURE: Conditional Compilation
        // Dieser Code läuft NUR im Browser, NICHT auf dem Server!
        #[cfg(feature = "hydrate")]
        {
            // 📦 WASM IMPORTS: Browser APIs nur hier importieren
            use web_sys::{MessageEvent, WebSocket};    // Browser WebSocket API
            use wasm_bindgen::closure::Closure;        // JavaScript Callbacks  
            use wasm_bindgen::JsCast;                  // Type Casting für WASM
            // 🚀 Nur einmal verbinden (wenn noch keine Daten da sind)
            if prices.read().is_empty() {
                
                // 🌐 BROWSER API: Window-Objekt holen
                let window = web_sys::window().expect("window");
                let location = window.location();
                
                // 📍 AUTOMATISCHE SERVER-ERKENNUNG: Host von aktueller URL holen
                // Fallback zu localhost:3000 falls es nicht funktioniert  
                let host = location.host().unwrap_or_else(|_| "127.0.0.1:3000".into());
                
                // 🔒 PROTOKOLL-ERKENNUNG: wss für HTTPS, ws für HTTP
                let protocol = location
                    .protocol()
                    .ok()
                    .filter(|p| p.starts_with("https"))  // Ist es HTTPS?
                    .map(|_| "wss")                      // Dann sicherer WebSocket
                    .unwrap_or("ws");                    // Sonst normaler WebSocket
                    
                // 🔗 WebSocket URL zusammenbauen: "ws://127.0.0.1:3000/ws"  
                let ws_url = format!("{}://{}/ws", protocol, host);
                
                // 📡 WEBSOCKET VERBINDUNG aufbauen
                if let Ok(ws) = WebSocket::new(&ws_url) {
                    
                    // 💾 TRICK: WebSocket im Browser-Window speichern
                    // Warum? Damit andere Funktionen (z.B. Button-Clicks) darauf zugreifen können!
                    if let Some(win) = web_sys::window() {
                        let _ = js_sys::Reflect::set(
                            win.as_ref(), 
                            &js_sys::JsString::from("__leptos_ws"), 
                            ws.as_ref()
                        );
                    }
                    // ═══════════════════════════════════════════════════════════════════════
                    // 📨 MESSAGE TYPES - Welche Daten können vom Server kommen?
                    // ═══════════════════════════════════════════════════════════════════════
                    
                    // 🔧 RUST FEATURE: Derive Macros
                    // #[derive(Deserialize)] = "Automatisch JSON → Rust Struct konvertieren"
                    #[derive(Deserialize)]
                    
                    // 📡 SERDE FEATURE: Tagged Union  
                    // #[serde(tag = "type")] bedeutet: JSON hat "type" field das bestimmt welcher Typ
                    // Beispiel: {"type": "price", "symbol": "BTC", "price": 50000}
                    #[serde(tag = "type")]
                    enum Msg {
                        // 💰 PREIS-UPDATE: Neuer Marktpreis für ein Symbol
                        #[serde(rename = "price")] 
                        Price { 
                            symbol: String,   // "BTCUSD"
                            price: f64,       // 50000.50  
                            volume: u64,      // Handelsvolumen
                            ts: i64           // Timestamp 
                        },
                        
                        // 🔄 TRADE-EXECUTION: Ein Handel wurde ausgeführt
                        #[serde(rename = "trade")] 
                        Trade { 
                            symbol: String,   // "BTCUSD"
                            price: f64,       // Ausführungspreis
                            size: f64,        // Handelsgröße
                            side: String,     // "buy" oder "sell"
                            ts: i64 
                        },
                        
                        // 📖 ORDERBOOK: Aktuelle Bid/Ask Preise
                        #[serde(rename = "book")] 
                        Book { 
                            symbol: String,
                            bids: Vec<(f64, f64)>,  // Kaufangebote: (Preis, Menge)
                            asks: Vec<(f64, f64)>,  // Verkaufsangebote: (Preis, Menge)
                            ts: i64 
                        },
                        
                        // 🖥️ SYSTEM-METRIKEN: Server Performance Info
                        #[serde(rename = "system")] 
                        System { 
                            cpu_pct: f64,     // CPU Auslastung %
                            mem_mb: u64,      // RAM Verbrauch MB
                            msg_rate: u64,    // Messages pro Sekunde  
                            ts: i64 
                        },
                        
                        // 🤷 FALLBACK: Unbekannte Message-Typen ignorieren
                        #[serde(other)] Other,
                    }
                    // ═══════════════════════════════════════════════════════════════════════
                    // 📡 MESSAGE HANDLER - Was passiert bei neuen Server-Nachrichten?
                    // ═══════════════════════════════════════════════════════════════════════
                    
                    // 🔧 WASM FEATURE: Closure (JavaScript Callback in Rust)
                    // Das ist kompliziert! Rust-Funktionen können nicht direkt als JS-Callbacks 
                    // verwendet werden. Closure::wrap() macht die Konvertierung.
                    let onmessage = Closure::wrap(Box::new({
                        
                        // 🔄 RUST FEATURE: Clone für Move-Closure  
                        // Wir müssen alle RwSignals "clonen" damit sie in die Closure "moved" werden können
                        // .clone() bei RwSignal kopiert nur den Pointer, nicht die Daten!
                        let prices = prices.clone();           
                        let trades = trades.clone();
                        let book_depth = book_depth.clone();
                        let msg_rate_sig = msg_rate.clone();
                        let latency_values = latency_values.clone();
                        let msg_count = msg_count.clone();
                        let msg_rate_timer = msg_rate_timer.clone();
                        let sample_max = sample_max.clone();
                        
                        // 🚀 CALLBACK FUNCTION: Wird bei jeder WebSocket-Message aufgerufen
                        // |e: MessageEvent| ist die Rust-Syntax für JavaScript-Callbacks
                        move |e: MessageEvent| {
                            
                            // ⏱️ PERFORMANCE MONITORING: Empfangszeit messen
                            let t_recv = web_sys::window().unwrap().performance().unwrap().now();
                            
                            // 📊 MESSAGE COUNTER: Anzahl empfangener Nachrichten erhöhen
                            msg_count.update(|c| *c += 1);
                            
                            // 🔄 MOMENTANE MESSAGE RATE BERECHNUNG (Client-seitig!)
                            let last_time = *msg_rate_timer.read();
                            if last_time == 0.0 {
                                *msg_rate_timer.write() = t_recv;
                            } else if t_recv - last_time >= 1000.0 {
                                let current_rate = *msg_count.read() as f64 * 1000.0 / (t_recv - last_time);
                                msg_rate_sig.update(|v| {
                                    v.push(current_rate);
                                    let cap = *sample_max.read();
                                    let extra = v.len().saturating_sub(cap);
                                    if extra > 0 { v.drain(0..extra); }
                                });
                                *msg_count.write() = 0;
                                *msg_rate_timer.write() = t_recv;
                            }
                            // 📄 JSON PARSING: WebSocket-Message zu Rust-Struct konvertieren
                            if let Some(txt) = e.data().as_string() {
                                // 🔧 SERDE FEATURE: JSON String → Msg Enum  
                                if let Ok(msg) = serde_json::from_str::<Msg>(&txt) {
                                    
                                    // 🔀 RUST FEATURE: Pattern Matching
                                    // Wie switch/case aber viel mächtiger! Jeder Message-Typ bekommt eigene Behandlung
                                    match msg {
                                        
                                        // 💰 PREIS-UPDATE verarbeiten
                                        Msg::Price { symbol, price, .. } => {
                                            // 🔄 LEPTOS FEATURE: Signal Update
                                            // prices.update() triggert automatisch UI-Neuzeichnung!
                                            prices.update(|map| {
                                                // 📊 RUST FEATURE: HashMap Entry API
                                                let entry = map.entry(symbol).or_insert_with(Vec::new);
                                                entry.push(price);  // Neuen Preis hinzufügen
                                                
                                                // 🧹 MEMORY MANAGEMENT: Alte Daten löschen (Performance!)
                                                let cap = *sample_max.read();
                                                if entry.len() > cap { 
                                                    entry.drain(0..entry.len() - cap); 
                                                }
                                            });
                                            
                                            // ⏱️ LATENCY MEASUREMENT: Wie schnell wird UI aktualisiert?
                                            let latency_values = latency_values.clone();
                                            
                                            // 🖼️ BROWSER API: requestAnimationFrame für präzise Latenz-Messung
                                            // Misst Zeit von "Daten empfangen" bis "UI gezeichnet"
                                            let cb = Closure::wrap(Box::new(move |_: f64| {
                                                let t_paint = web_sys::window().unwrap().performance().unwrap().now();
                                                let dt = t_paint - t_recv;  // Latenz berechnen
                                                
                                                // 📊 Latenz-Wert speichern für Charts
                                                let mut lv = latency_values.write();
                                                lv.push(dt);
                                                let cap = *sample_max.read();
                                                let extra = lv.len().saturating_sub(cap);
                                                if extra > 0 { lv.drain(0..extra); }
                                            }) as Box<dyn FnMut(f64)>);
                                            
                                            let _ = web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
                                            cb.forget();
                                        }
                                        Msg::Trade { symbol, price, side, .. } => {
                                            trades.update(|t| {
                                                t.push((symbol, price, side));
                                                if t.len() > 100 { t.drain(0..t.len() - 100); }
                                            });
                                        }
                                        Msg::Book { symbol, bids, asks, .. } => {
                                            book_depth.update(|map| {
                                                let bid_prices: Vec<f64> = bids.iter().map(|(p, _)| *p).collect();
                                                let ask_prices: Vec<f64> = asks.iter().map(|(p, _)| *p).collect();
                                                map.insert(symbol, (bid_prices, ask_prices));
                                            });
                                        }
                                        Msg::System { .. } => {
                                            // System-Messages ignorieren - wir berechnen Rate client-seitig!
                                        }
                                        Msg::Other => {}
                                    }
                                }
                            }
                        }
                    }) as Box<dyn FnMut(_)>);
                    let _ = ws.add_event_listener_with_callback("message", onmessage.as_ref().unchecked_ref());
                    onmessage.forget(); // leak closure for lifetime of page
                }
            }
        }
    });

    // FPS measurement via rAF, record rolling FPS once per second
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            use std::rc::Rc;
            use std::cell::RefCell;
            let start_sec = RwSignal::new(web_sys::window().unwrap().performance().unwrap().now());
            let frames = RwSignal::new(0u32);
            let fps_values_signal = fps_values.clone();
            let sample_max_signal = sample_max.clone();

            let cb_cell: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
            let cb_cell_clone = cb_cell.clone();
            let raf_cb = Closure::wrap(Box::new(move |_: f64| {
                frames.update(|f| *f += 1);
                let now = web_sys::window().unwrap().performance().unwrap().now();
                let elapsed = now - *start_sec.read();
                if elapsed >= 1000.0 {
                    let fps = *frames.read() as f64 * 1000.0 / elapsed.max(1.0);
                    {
                        let mut v = fps_values_signal.write();
                        v.push(fps);
                        let cap = *sample_max_signal.read();
                        let extra = v.len().saturating_sub(cap);
                        if extra > 0 { v.drain(0..extra); }
                    }
                    *start_sec.write() = now;
                    *frames.write() = 0;
                }
                if let Some(cb) = cb_cell_clone.borrow().as_ref() {
                    let _ = web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
                }
            }) as Box<dyn FnMut(f64)>);
            *cb_cell.borrow_mut() = Some(raf_cb);
            if let Some(cb) = cb_cell.borrow().as_ref() {
                let _ = web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
            }
            // Intentionally leak closure so it runs for app lifetime
            cb_cell.borrow_mut().take().unwrap().forget();
        }
    });

    fn sparkline_points(data: &[f64], width: f64, height: f64) -> String {
        if data.is_empty() { return String::new(); }
        let (min, max) = data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| (mn.min(v), mx.max(v)));
        let range = if (max - min).abs() < 1e-9 { 1.0 } else { max - min };
        let n = data.len() as f64;
        let step = if n > 1.0 { width / (n - 1.0) } else { width };
        let mut out = String::new();
        for (i, &v) in data.iter().enumerate() {
            let x = step * (i as f64);
            let y = height - ((v - min) / range) * height;
            if i > 0 { out.push(' '); }
            out.push_str(&format!("{:.1},{:.1}", x, y));
        }
        out
    }

    fn stats(data: &[f64]) -> (f64, f64, f64) {
        if data.is_empty() { return (0.0, 0.0, 0.0); }
        let mean = data.iter().sum::<f64>() / (data.len() as f64);
        let mut v = data.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx50 = ((v.len() - 1) as f64 * 0.50).round() as usize;
        let idx95 = ((v.len() - 1) as f64 * 0.95).round() as usize;
        (mean, v[idx50], v[idx95])
    }

    view! {
        <Stylesheet id="leptos" href="/pkg/rust-leptos-sandbox.css"/>
        <Title text="Leptos Live Data Performance Test"/>

        <Router>
            <main style="padding: 1rem; font-family: system-ui;">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
                
                <h1>"🚀 Real-time Market Data Stream"</h1>
                <p style="color: #666;">
                    {move || format!("Total messages: {} | FPS: {:.1}", 
                        *msg_count.read(), 
                        fps_values.read().last().cloned().unwrap_or(0.0)
                    )}
                </p>

                // Price charts for each symbol
                <section style="margin: 2rem 0;">
                    <h2>"📈 Live Price Feeds"</h2>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                        {move || {
                            let p = prices.read();
                            let mut symbols: Vec<_> = p.keys().cloned().collect();
                            symbols.sort();
                            symbols.into_iter().map(|symbol| {
                                let data = p.get(&symbol).cloned().unwrap_or_default();
                                let latest = data.last().cloned().unwrap_or(0.0);
                                let (mean, p50, p95) = stats(&data);
                                view! {
                                    <div style="border:1px solid #ddd;padding:0.5rem;border-radius:4px;">
                                        <h3 style="margin:0 0 0.5rem 0;font-size:1rem;">{symbol.clone()}</h3>
                                        <p style="margin:0;font-size:1.5rem;font-weight:bold;color:#0066cc;">
                                            {format!("${:.2}", latest)}
                                        </p>
                                        <svg width="100%" height="60" viewBox="0 0 300 60" style="margin-top:0.5rem;">
                                            <polyline stroke="#0066cc" fill="none" stroke-width="2"
                                                points={sparkline_points(&data, 300.0, 60.0)} />
                                        </svg>
                                        <p style="margin:0.5rem 0 0 0;font-size:0.75rem;color:#888;">
                                            {format!("μ:{:.2} p50:{:.2} p95:{:.2}", mean, p50, p95)}
                                        </p>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </div>
                </section>

                // Recent trades feed
                <section style="margin: 2rem 0;">
                    <h2>"💱 Recent Trades"</h2>
                    <div style="max-height:200px;overflow-y:auto;border:1px solid #ddd;padding:0.5rem;border-radius:4px;font-family:monospace;font-size:0.85rem;">
                        {move || {
                            let t = trades.read();
                            t.iter().rev().take(20).map(|(symbol, price, side)| {
                                let color = if side == "buy" { "#00cc66" } else { "#ff6666" };
                                view! {
                                    <div style=format!("padding:0.25rem;border-bottom:1px solid #f0f0f0;color:{}", color)>
                                        {format!("{} ${:.2} {}", symbol, price, side.to_uppercase())}
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </div>
                </section>

                // Order book depth
                <section style="margin: 2rem 0;">
                    <h2>"📊 Order Book Depth"</h2>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                        {move || {
                            let books = book_depth.read();
                            let mut symbols: Vec<_> = books.keys().cloned().collect();
                            symbols.sort();
                            symbols.into_iter().map(|symbol| {
                                let (bids, asks) = books.get(&symbol).cloned().unwrap_or_default();
                                view! {
                                    <div style="border:1px solid #ddd;padding:0.5rem;border-radius:4px;font-family:monospace;font-size:0.8rem;">
                                        <h3 style="margin:0 0 0.5rem 0;font-size:0.9rem;">{symbol.clone()}</h3>
                                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.5rem;">
                                            <div>
                                                <strong style="color:#00cc66;">"BIDS"</strong>
                                                {bids.iter().take(5).map(|p| view! {
                                                    <div>{format!("${:.2}", p)}</div>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                            <div>
                                                <strong style="color:#ff6666;">"ASKS"</strong>
                                                {asks.iter().take(5).map(|p| view! {
                                                    <div>{format!("${:.2}", p)}</div>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </div>
                </section>

                // Performance metrics
                <section style="margin: 2rem 0;">
                    <h2>"⚡ Performance Metrics"</h2>
                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;">
                        <div style="border:1px solid #ddd;padding:0.5rem;border-radius:4px;">
                            <h3 style="margin:0 0 0.5rem 0;font-size:1rem;">"💬 Momentane Message Rate (msg/s)"</h3>
                            <p style="margin:0;font-size:1.5rem;font-weight:bold;color:#9933ff;">
                                {move || {
                                    let current = msg_rate.read().last().cloned().unwrap_or(0.0);
                                    format!("{:.1}", current)
                                }}
                            </p>
                            <div style="font-size:0.75rem;color:#666;margin-top:0.25rem;">
                                {move || {
                                    let rates = msg_rate.read();
                                    if rates.len() >= 2 {
                                        let current = rates.last().cloned().unwrap_or(0.0);
                                        let previous = rates[rates.len()-2];
                                        let change = current - previous;
                                        let trend = if change > 0.5 { "📈" } else if change < -0.5 { "📉" } else { "➡️" };
                                        format!("{} {:.1} vs vorige Sekunde", trend, change)
                                    } else {
                                        "Warte auf Daten...".to_string()
                                    }
                                }}
                            </div>
                            <svg width="100%" height="60" viewBox="0 0 300 60" style="margin-top:0.5rem;">
                                <polyline stroke="#9933ff" fill="none" stroke-width="2"
                                    points={move || sparkline_points(&msg_rate.read(), 300.0, 60.0)} />
                            </svg>
                        </div>
                        <div style="border:1px solid #ddd;padding:0.5rem;border-radius:4px;">
                            <h3 style="margin:0 0 0.5rem 0;font-size:1rem;">"Render FPS"</h3>
                            <p style="margin:0;font-size:1.5rem;font-weight:bold;">
                                {move || format!("{:.1}", fps_values.read().last().cloned().unwrap_or(0.0))}
                            </p>
                            <svg width="100%" height="60" viewBox="0 0 300 60" style="margin-top:0.5rem;">
                                <polyline stroke="#00cc66" fill="none" stroke-width="2"
                                    points={move || sparkline_points(&fps_values.read(), 300.0, 60.0)} />
                            </svg>
                        </div>
                        <div style="border:1px solid #ddd;padding:0.5rem;border-radius:4px;">
                            <h3 style="margin:0 0 0.5rem 0;font-size:1rem;">"Latency (ms)"</h3>
                            <p style="margin:0;font-size:1.5rem;font-weight:bold;">
                                {move || {
                                    let d = latency_values.read();
                                    format!("{:.2}", d.last().cloned().unwrap_or(0.0))
                                }}
                            </p>
                            <svg width="100%" height="60" viewBox="0 0 300 60" style="margin-top:0.5rem;">
                                <polyline stroke="#ff6666" fill="none" stroke-width="2"
                                    points={move || sparkline_points(&latency_values.read(), 300.0, 60.0)} />
                            </svg>
                            <p style="margin:0.5rem 0 0 0;font-size:0.75rem;color:#888;">
                                {move || {
                                    let d = latency_values.read();
                                    let (mean, p50, p95) = stats(&d);
                                    format!("μ:{:.2} p50:{:.2} p95:{:.2}", mean, p50, p95)
                                }}
                            </p>
                        </div>
                    </div>
                </section>

                // Control panel
                <section style="margin: 2rem 0;padding:1rem;background:#f5f5f5;border-radius:4px;">
                    <h2 style="margin:0 0 1rem 0;">"🎛️ Control Panel"</h2>
                    <div style="display:flex;gap:1rem;flex-wrap:wrap;align-items:center;">
                        <div>
                            <label for="freq" style="display:block;margin-bottom:0.25rem;font-size:0.9rem;">"Update Frequency (ms)"</label>
                            <input id="freq" type="number" value=50 min=10 max=1000 step=10
                                style="padding:0.5rem;border:1px solid #ccc;border-radius:4px;"
                                on:change=move |ev| {
                                    #[cfg(feature = "hydrate")]
                                    {
                                        if let Some(win) = web_sys::window() {
                                            if let Ok(val) = event_target_value(&ev).parse::<u64>() {
                                                if let Ok(js_ws) = js_sys::Reflect::get(win.as_ref(), &js_sys::JsString::from("__leptos_ws")) {
                                                    if !js_ws.is_undefined() {
                                                        if let Ok(ws) = js_ws.dyn_into::<web_sys::WebSocket>() {
                                                            let _ = ws.send_with_str(&format!("{{\"frequency_ms\":{}}}", val));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } />
                        </div>
                        <div>
                            <label for="sample" style="display:block;margin-bottom:0.25rem;font-size:0.9rem;">"Sample Window"</label>
                            <select id="sample" 
                                style="padding:0.5rem;border:1px solid #ccc;border-radius:4px;"
                                on:change=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<usize>() { *sample_max.write() = val; }
                                }>
                                <option value="200" selected>"200"</option>
                                <option value="500">"500"</option>
                                <option value="1000">"1000"</option>
                            </select>
                        </div>
                        <button 
                            style="padding:0.5rem 1rem;background:#ff6666;color:white;border:none;border-radius:4px;cursor:pointer;font-weight:bold;"
                            on:click=move |_| {
                                prices.write().clear();
                                trades.write().clear();
                                book_depth.write().clear();
                                msg_rate.write().clear();
                                fps_values.write().clear();
                                latency_values.write().clear();
                                *msg_count.write() = 0;
                                *msg_rate_timer.write() = 0.0;  // ← TIMER auch resetten!
                            }>
                            "Reset All Metrics"
                        </button>
                    </div>
                    <p style="margin:1rem 0 0 0;font-size:0.85rem;color:#666;">
                        "Lower frequency = higher message rate. Adjust to stress test frontend rendering performance."
                    </p>
                </section>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div style="text-align:center;padding:2rem;">
            <p style="font-size:1.1rem;color:#666;">
                "Monitoring live data streams. Charts will populate as messages arrive."
            </p>
        </div>
    }
}
