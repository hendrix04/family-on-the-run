# Technology options for a family dashboard

Research retrieved October 11, 2026. The user has clarified that frontends may diverge and that one hosted backend is desirable. Recommendations below are engineering judgments, separated from documented framework capabilities.

## Which architecture is the shortest useful starting path, and what does one binary mean?

### Takeaway

**Recommendation:** Start with one hosted Go program: standard `net/http`, embedded HTML/CSS and small vanilla JavaScript files, FullCalendar Standard for week/month views, and SQLite through `modernc.org/sqlite`. Serve the touchscreen browser and later phone clients from the same authoritative backend. This recommendation follows the user's priorities and Go's documented asset embedding and the driver's documented platform support. [Go embed](https://pkg.go.dev/embed), [Go HTTP server](https://pkg.go.dev/net/http), [SQLite driver](https://pkg.go.dev/modernc.org/sqlite), [FullCalendar vanilla integration](https://fullcalendar.io/docs/vanilla-js)

### Cited Findings

- `embed.FS` embeds files at compile time and works directly with `net/http` and `html/template`; the documentation includes a server example serving embedded files. This allows HTML, CSS, calendar JavaScript, icons, and templates to live in the delivered executable. [Go embed](https://pkg.go.dev/embed)
- `modernc.org/sqlite` is a CGo-free port, licensed BSD-3-Clause. Its listed targets include Windows amd64/arm64, macOS amd64/arm64, and Linux amd64/arm/arm64 among others. Its performance documentation explicitly says the transpiled engine is slower for CPU-bound workloads than the native C engine; no dashboard performance benchmark has been run. [Driver documentation and target table](https://pkg.go.dev/modernc.org/sqlite)
- FullCalendar Standard/non-premium plugins are MIT licensed; premium resource/scheduler plugins have different terms. Standard views include Month, TimeGrid, List and DayGrid, and the documentation provides a vanilla JavaScript integration. Touch support includes scrolling and long press to select, drag and resize. [License](https://fullcalendar.io/license), [Views](https://fullcalendar.io/docs), [Vanilla JS](https://fullcalendar.io/docs/vanilla-js), [Touch](https://fullcalendar.io/docs/touch)
- Go selects operating-system and CPU targets through `GOOS`/`GOARCH`; cross platform means producing platform-specific executable builds, rather than one executable that runs unchanged on Windows, macOS and Linux. [Go build environment](https://go.dev/doc/install/source#environment)
- Even native GUI frameworks use platform packaging: Tauri documents installers, macOS app bundles and mobile store builds, while Fyne's mobile tool produces an APK and an Apple app bundle. [Tauri distribution](https://tauri.app/distribute/), [Fyne mobile packaging](https://docs.fyne.io/started/mobile/)
- A SQLite database is an in-process database engine, without a separate database-server process. The driver still opens persistent database files; compiling code/assets into a binary does not put future mutable family data inside that immutable executable. [SQLite driver](https://pkg.go.dev/modernc.org/sqlite)

### Inferences

- The explicit frontend choice is **plain HTML/CSS + a small amount of vanilla JavaScript + FullCalendar**, not “Go UI.” Go and Rust as backends do not themselves avoid JavaScript in a browser frontend. Embed the dependency files locally instead of relying on a CDN. The cited embedding and vanilla integration make this possible.
- One server instance should own tasks, future recipes, grocery lists, calendar cache and provider connections. All clients should share its HTTP API. Keep Google OAuth tokens on that server in a persistent protected store; clients should use the project API rather than each becoming its own Google calendar integration.
- With the chosen CGo-free driver and embedded assets, a target build such as `CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -o family-dashboard ./cmd/family-dashboard` is a suitable future release-validation command. This is an example, not an executed command or an existing repository path.
- The browser is an external runtime dependency of this approach. There is no reason to ship Node.js with the product when the frontend consists of static vanilla JavaScript. The server binary can run on the touchscreen computer, another home machine, or a hosted machine; decide location after validating the first screen.
- “One distributed application executable” should allow a separate mutable database, tokens, backups and configuration. “One file ever on disk, no browser or operating-system libraries” is a different requirement and incompatible with normal persistent data and typical GUI packaging.

### Gaps

- Target deployment OS, CPU, always-on host, internet connectivity, local-only vs remote access, and device startup mechanism are unspecified.
- The selected driver and assets have not been compiled in this repository; no reproducibility, binary-size or startup benchmark is claimed.
- Google authorization/setup details are covered by the separate calendar researcher. The hosted-backend recommendation does not remove Google onboarding requirements.

## Which desktop/mobile alternatives are serious, and how do packaging and licenses differ?

### Takeaway

**Recommendation:** Native frontend selection can wait because the shared hosted API is the stable investment. The relevant alternatives beyond Java/Kotlin, JavaScript and Dart are Go native GUIs, C# Avalonia, Rust Dioxus/Slint/Tauri and C++ Qt; however several still render HTML through webviews. Documented capabilities and my suitability judgment follow.

### Cited Findings

| Option | Authored UI and documented targets | Binary/runtime/license constraints | Suitability judgment |
|---|---|---|---|
| **Go HTTP + embedded web UI** | HTML/CSS/JS in browsers; server targets Windows/macOS/Linux including common ARM targets with the chosen driver. [Go targets](https://go.dev/doc/install/source#environment), [driver targets](https://pkg.go.dev/modernc.org/sqlite) | Embedded assets; browser required; mutable data separate. Driver BSD-3-Clause; calendar Standard MIT. [Embed](https://pkg.go.dev/embed), [driver](https://pkg.go.dev/modernc.org/sqlite), [calendar license](https://fullcalendar.io/license) | **First choice for this project**, especially after accepting divergent frontends and one hosted backend. |
| **Go + Wails** | HTML/CSS/JS in native desktop window. Current v3 desktop Beta targets Windows/macOS/Linux amd64/arm64; Android/iOS explicitly experimental. [v3 status](https://v3.wails.io/status/) | Wails v2 bundles assets into one executable; Windows WebView2 and Linux GTK/WebKit runtime dependencies remain. [v2 introduction](https://wails.io/docs/introduction/), [installation](https://wails.io/docs/gettingstarted/installation/), [Linux runtime](https://v2.wails.io/docs/guides/linux-distro-support/) | Use v2 if an own desktop window is immediately necessary and Go is preferred; don't base required mobile delivery on experimental v3 support. |
| **Rust + Tauri 2** | Web frontend, native Rust application core; Windows/macOS/Linux/Android/iOS. [Tauri](https://v2.tauri.app/), [distribution](https://tauri.app/distribute/) | Uses OS webviews: WebView2, WKWebView, WebKitGTK and Android WebView. Platform installers/bundles; MIT/Apache-2.0 as applicable. [Webviews](https://v2.tauri.app/reference/webview-versions/), [license](https://github.com/tauri-apps/tauri#licenses) | Best immediate Rust option if shipping the same web-based client as installed desktop and mobile apps matters. Not necessary for the initial hosted dashboard. |
| **Rust + Dioxus** | Author components in Rust with HTML/CSS. Windows/macOS/Linux plus Android/iOS; web target also documented. [Project](https://github.com/DioxusLabs/dioxus), [mobile](https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/) | Desktop/mobile default uses webviews, rather than platform-native widgets; Linux requires WebKitGTK and xdotool, Windows WebView2. MIT/Apache-2.0. Native renderer is explicitly experimental. [Desktop](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/), [dependencies](https://dioxuslabs.com/learn/0.7/getting_started/), [project/license](https://github.com/DioxusLabs/dioxus) | Interesting if Rust-authored UI is a positive goal. The calendar library could still require JavaScript interop; avoid claiming it removes browser machinery or is a faster MVP. |
| **Go + Fyne** | Go widget API, single shared desktop/mobile codebase. Android/iOS packaging documented, including macOS/Xcode build requirements for iOS. [Project](https://github.com/fyne-io/fyne), [mobile](https://docs.fyne.io/started/mobile/) | Desktop resources can be bundled into executable; build requires C compiler/system graphics development tools. BSD-3-Clause. [Bundling](https://docs.fyne.io/extend/bundle/), [prerequisites/license](https://github.com/fyne-io/fyne) | Best straightforward Go alternative for avoiding HTML/JS UI. Built-in Calendar is a date/time picker, so a family event grid remains custom UI work. [Calendar widget](https://docs.fyne.io/api/v2/widget/calendar/) |
| **C# + Avalonia** | C#/XAML GUI; Windows/macOS/Linux/iOS/Android/WebAssembly, with documented tier-specific version support, including Linux ARM and embedded Linux. [Support matrix](https://docs.avaloniaui.net/docs/supported-platforms) | MIT; Native AOT documented, requiring compiled bindings and AOT-compatible dependencies. Linux still needs platform libraries. “No .NET runtime install” is not “no native dependencies.” [AOT](https://docs.avaloniaui.net/docs/deployment/native-aot), [license](https://avaloniaui.net/avalonia/windows/), [Linux packages](https://docs.avaloniaui.net/docs/supported-platforms) | Strongest general C# native GUI candidate here; a calendar-week/month event renderer is a separate component-selection task. |
| **C++ + Qt Quick** | C++/QML; Windows/macOS/Linux, Android and iOS, including ARM targets documented per platform. [Support matrix](https://doc.qt.io/qt-6/supported-platforms.html) | LGPLv3/GPLv3/commercial depending on modules; static distribution must preserve the applicable relinking/source obligations. Current Linux ARM desktop documentation uses Raspberry Pi 5/Ubuntu as reference platform and notes support scope. [Licensing](https://doc.qt.io/qt-6/licensing.html), [obligations](https://www.qt.io/development/open-source-lgpl-obligations), [targets](https://doc.qt.io/qt-6/supported-platforms.html) | Consider for strong existing Qt/C++ experience or appliance hardware needs. Least compelling shortest path for this user's first Home screen. |

- **Slint** is a Rust/C++ declarative GUI alternative. Its Royalty-Free license permits desktop/mobile/web applications with attribution, but defines those as general-purpose computers/devices; embedded-device use needs separate consideration. GPLv3 is an alternative. Its currently retrievable mobile documentation lists Android/Rust support and iOS planned. [License text](https://slint.dev/agreements/slint-royalty-free-license.pdf), [project licenses](https://github.com/slint-ui/slint/blob/master/LICENSE.md), [mobile documentation](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/)
- **Gio** is another Go UI option supporting desktop/mobile/Wasm, requiring platform window/input/GPU libraries and a separate packaging tool. It does not improve the hosted-backend decision. [Install/platform list](https://gioui.org/doc/install)
- **.NET MAUI** officially lists Windows, macOS through Mac Catalyst, Android and iOS; Linux is absent from Microsoft's support list. That is a reason to prefer Avalonia if Linux is mandatory. [Microsoft supported platforms](https://learn.microsoft.com/en-us/dotnet/maui/supported-platforms)
- **Flutter baseline:** Flutter's official Windows distribution instructions require the executable, DLL files, a data directory and Visual C++ runtime support; a zip/installer containing them is not a lone executable. If the binary requirement now applies to the backend, Flutter becomes an eligible future client rather than being excluded by that backend requirement. [Flutter Windows distribution](https://docs.flutter.dev/platform-integration/windows/building)
- **Python baseline:** PyInstaller supports one-file distribution by packing modules/native files into the executable and extracting supporting files into a temporary directory at startup. This is packaging rather than a Go-style directly compiled server binary. [PyInstaller operating mode](https://pyinstaller.org/en/stable/operating-mode.html)
- **.NET single-file and Native AOT are separate deployment concepts.** The normal single-file documentation says native libraries need an explicit extraction setting; Avalonia AOT says no installed .NET runtime is required but advises verifying third-party AOT compatibility. Do not claim the extraction setting guarantees one-file Native AOT Avalonia. [Microsoft single-file](https://learn.microsoft.com/et-ee/dotnet/core/deploying/single-file/overview), [Avalonia AOT](https://docs.avaloniaui.net/docs/deployment/native-aot)

### Inferences

- A shared HTTP API means desktop and mobile clients can be selected on their own usability merits later. Native Swift/Kotlin phones do not force a rewrite of a Go backend, nor does a Go server dictate a Go desktop GUI.
- If strict own-window packaging is chosen now, Wails/Tauri preserve the ability to use FullCalendar. Fyne/Avalonia/Qt/Slint would require evaluating or implementing a comparable calendar event UI; a date picker alone does not satisfy week/month event management.
- I would not start by sharing an autonomous local database among native clients. One household server is easier to reason about for task ownership, meal edits and calendar provider credentials. Autonomous offline phones create synchronization and conflict-resolution work regardless of GUI language.

### Gaps

- Slint's mobile search result has an older crawl date; iOS status should be rechecked directly before making a product commitment. It is not shortlisted because this source currently does not establish full iOS support.
- No Avalonia/Qt native event-calendar component was evaluated sufficiently to call it a drop-in alternative to FullCalendar.
- No framework packaging was experimentally built, so strict one physical executable claims beyond cited docs need a proof build.

## Should desktop/mobile share UI, and what should be built first?

### Takeaway

**Recommendation:** Share the hosted backend and data model now, use one responsive web client first, and allow the phone frontend to diverge when there is a clear need for native platform features. The Shopify example is current and real, but it is evidence about Shopify's economics, not a requirement to produce two native clients for this household dashboard.

### Cited Findings

- Shopify's September 10, 2026 article announces migrating all mobile apps from React Native back to Swift/Kotlin. Shop has already shipped fully native, the Shopify app is underway, others will follow. The stated reason is that coding agents reduced implementation/parity cost, while platform-specific tooling/capabilities remain advantageous; the article also says maintaining two platforms still costs effort and React Native apps can be fast. [Shopify announcement](https://shopify.engineering/back-to-native)
- This supersedes the older January 2025 Shopify article, which reported completing all-app migration to React Native. Neither should be quoted without its date. [Older Shopify report](https://shopify.engineering/five-years-of-react-native-at-shopify), [newer announcement](https://shopify.engineering/back-to-native)
- CSS pixels are not necessarily physical pixels; `devicePixelRatio` measures their ratio and page zoom affects it. A 2560×1440 panel therefore does not establish the CSS viewport size. [MDN devicePixelRatio](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio)
- WCAG 2.2's target-size-minimum criterion specifies 24×24 CSS pixels with stated exceptions, while the enhanced criterion specifies 44×44 CSS pixels with exceptions. [W3C minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum), [W3C enhanced](https://www.w3.org/WAI/WCAG22/Understanding/target-size-enhanced)

### Inferences

1. **Proof of use:** Build the left tabs and five Home columns with a daily agenda and four real local task lists. Start with viewing existing calendars and completing/adding tasks. Design person records configurable for an open source household project; seed Lindsay, Matt, Carter and Sydney for this user.
2. **Calendar integration:** Add one backend Google connection flow, calendar selection, colored merged events, week/month views, a cached last-known result and a visible last-refresh time. Defer editing Google events unless it is needed to use the dashboard.
3. **Hosted sharing:** Reuse the same HTTP API and authoritative SQLite store for a responsive phone web view. Add household authentication when reachable from multiple devices or the internet; validate shared updates across two clients. This adds client access, rather than a second database and second provider integration.
4. **Food/House:** Make the chore page a static recurring schedule initially. Add one useful food workflow at a time: first a meal plan and manual grocery list, then recipe storage/import as needed.
5. **Native divergence later:** Choose Swift/Kotlin, Flutter, Avalonia, Dioxus or another client only after identifying a need that the browser frontend does not meet. Shared semantics/API/schema are more valuable here than forcing identical UI code.
6. **Touch layout:** Prefer 48–56 CSS-pixel primary controls as a design recommendation, beyond WCAG's 44-pixel enhanced benchmark, with generous spacing and tested on the actual display. At an assumed 150% scale, approximate logical width is `2560 / 1.5 = 1706.67`; budget the sidebar and five columns against measured `window.innerWidth`, not panel resolution. Check readable text and independent column scrolling before building the rest of Food.
7. **Project boundary:** Stay with a single household server process and a handful of database tables/HTTP routes. The user wants an application, not a plug-in host, home-automation framework, microservices system or offline synchronization project.

### Gaps

- Offline mobile operation, notifications, remote access, barcode scanning, camera imports and app-store distribution have not been requested. Those may eventually justify native phone clients and/or synchronization, but should not shape the first implementation.
- The display's actual operating-system scale, browser zoom and wall viewing distance are not specified. The 150% calculation is an illustration with an explicit assumption, not a measured viewport.

