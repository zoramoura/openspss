# Self-Hosted SPSS Alternative: Modern Statistical Computing Platform

A privacy-first, reactive, open-source alternative to IBM SPSS Statistics. Built with a high-performance Rust and WebAssembly computation engine, a Vue 3 and Tailwind CSS frontend, and a lightweight Tauri desktop container.

---

## 1. Project Overview and Strategic Moat

Legacy statistical software suites such as IBM SPSS Statistics remain entrenched across academic, healthcare, and enterprise research environments despite significant operational liabilities. These traditional tools impose steep annual enterprise licensing fees that frequently reach thousands of dollars per seat, burdening universities, clinical research teams, and small-to-medium enterprises. Architecturally, legacy SPSS relies on a 1990s desktop paradigm characterized by disconnected floating windows, non-reactive recalculation loops, and installation footprints exceeding 1 gigabyte. Furthermore, they lack web-native multi-user collaboration and modern data pipeline integration.

This project delivers an open-source, self-hosted, and web-native analytical environment that directly addresses these vulnerabilities through three foundational pillars:

*   **Airtight Compliance and Data Sovereignty:** Highly regulated research domains subject to HIPAA and GDPR restrictions cannot transmit raw participant or patient records to third-party cloud analytics services. By executing all computations locally within the client browser sandbox or desktop runtime via WebAssembly, raw datasets never transit external networks.
*   **Zero-Footprint Deployment:** Users can launch full analytical workflows directly through any modern browser without local administrative installation rights. For desktop environments, Tauri provides lightweight native installers of approximately 15 megabytes, compared to the 1-gigabyte footprint of legacy distributions.
*   **Live Reactive Calculations:** Traditional statistical workflows force users to execute modal dialogues repeatedly whenever raw records are cleaned or filtered. This platform introduces a reactive computational pipeline where descriptive metrics and inferential models update dynamically as data values change.

The following comparison illustrates the architectural shift between legacy SPSS and this modern open-source platform across foundational operational dimensions.

| Operational Dimension | Legacy IBM SPSS Statistics | Modern Self-Hosted Alternative |
| :--- | :--- | :--- |
| **Licensing and Cost** | Closed enterprise tiers (\$thousands/seat/year) | Permissive open-source self-hosted deployment |
| **Execution Environment** | Monolithic Java and C++ desktop runtime | Client-side Rust compiled to WebAssembly (WASM) |
| **Data Privacy Model** | Local install or remote enterprise server transfer | Local-first; zero raw data transit over networks |
| **Installer Footprint** | Heavy package (> 1 GB) | Ultralight desktop build (~15 MB via Tauri) or zero-install browser access |
| **Workspace Ergonomics** | Isolated floating windows and modal popups | Unified split-screen reactive workspace with side drawers |
| **Recalculation Model** | Static manual rerun after data modifications | Reactive live pipeline updates across views |

### Critical Engineering Roadblocks

To displace legacy software effectively, this project directly addresses three major technical challenges:

1.  **SPSS Binary File Compatibility:** Full real-world adoption requires robust parsing and export of proprietary `.sav` and `.zsav` binary files, preserving variable labels, value label mappings, user-defined missing values, and measurement level metadata.
2.  **Strict Statistical Validation:** Numerical algorithms must match standard scientific benchmarks to at least 4 decimal places under complex edge conditions, including unbalanced factorial designs, Type III Sums of Squares, and missing data exclusions.
3.  **Syntax Automation Layer:** Legacy power users and institutional pipelines depend on scriptable automation; future iterations must provide an accessible syntax scripting layer to preserve automated research workflows.

---

## 2. System Architecture and Technology Stack

The platform is engineered as a decoupled, multi-tier system that isolates heavy matrix calculations from interface rendering. This design preserves a consistent 60 frames-per-second refresh rate during intensive dataset manipulations and complex numerical evaluations.

### 2.1 Compute Engine: Rust and WebAssembly

The computational foundation is implemented in Rust and compiled to WebAssembly (`wasm32-unknown-unknown`) using `wasm-pack`. Rather than incurring the heavy download overhead and cold-start latencies of embedded Python runtimes (such as Pyodide at 20 to 35 megabytes compressed) or R runtimes (such as WebR), the custom Rust engine yields a compiled binary payload under 2 megabytes with instantaneous startup.

*   **Linear Algebra and Matrices:** Powered by `nalgebra` and `ndarray` for matrix decompositions, multidimensional indexing, and ordinary least squares estimation.
*   **Probability Distributions:** Powered by `statrs` for continuous probability density functions, cumulative distribution functions, and cumulative p-value computations across Student's t, Snedecor's F, and normal distributions.
*   **Columnar Memory and Data Layout:** High-volume transformations leverage Apache Arrow columnar record batches, passing typed memory slices (`Float64Array`, `Int32Array`) directly across the JavaScript and WebAssembly runtime boundary to bypass JSON serialization overhead.
*   **WASM Bindings:** Integrated using `wasm-bindgen` and `serde-wasm-bindgen` for zero-copy memory views and structured output generation.

### 2.2 Client Interface: Vue 3 and Virtualized Canvas Grid

The user interface is built on Vue 3 utilizing the Composition API, TypeScript, Pinia for centralized state management, and Tailwind CSS for interface styling.

*   **Virtualized Data Grid:** Grid rendering utilizes a high-performance canvas-based data grid (incorporating Glide Data Grid wrappers) to smoothly virtualize millions of cells without degrading DOM responsiveness.
*   **Dedicated Worker Abstraction:** To guarantee interface fluidity, the WebAssembly module executes within a dedicated Web Worker managed via `comlink`. The main thread posts column memory references to the worker, which triggers Rust calculation routines and emits structured results asynchronously.

### 2.3 Cross-Platform Desktop Strategy: Tauri vs. Alternatives

The desktop edition wraps the shared web assets inside a Tauri shell rather than competing frameworks:

*   **Why Tauri over Electron:** Electron bundles an entire Chromium runtime and Node.js instance, resulting in executables over 150 megabytes and sluggish performance when rendering massive tabular datasets on low-spec client machines. Tauri leverages the operating system's native webview alongside a lightweight Rust backend, achieving a compact 15-megabyte distribution and minimal memory consumption.
*   **Why Tauri over Flutter:** While Flutter provides native cross-platform rendering, its spreadsheet and virtualized data table ecosystem is far less mature for advanced analytics compared to web-native canvas grids like Glide Data Grid. Furthermore, executing Python or R data science interop in Flutter requires complex native C FFI bindings.

---

## 3. Core UX and SPSS Mental Model

The application preserves the familiar mental model of traditional statistical analysis while replacing obsolete interface mechanics with unified modern ergonomics.

### 3.1 Dual-View Spreadsheet Interface

Data modeling mirrors the canonical two-faced spreadsheet structure required by empirical social science and clinical researchers:

*   **Data View:** A tabular grid where rows represent observed cases (participants or records) and columns represent experimental variables.
*   **Variable View:** A metadata dictionary where each row defines an individual column's configuration. Variables are configured across the following mandatory parameters:
    *   **Name and Label:** Standard variable identifier alongside a descriptive human-readable label.
    *   **Data Type:** Numeric, string, or date formatting.
    *   **Measurement Level:** Strict categorization into Nominal, Ordinal, or Scale (interval/ratio), which downstream analysis dialogs inspect to restrict valid tests.
    *   **Value Labels:** Mapping integer codes to categorical labels (for example, mapping 0 to Male and 1 to Female).
    *   **Missing Values:** User-defined discrete missing values excluded from statistical aggregations.

### 3.2 Unified Split-Screen and Side Drawers

Traditional SPSS workflows suffer from window sprawl, requiring users to manage separate floating windows for datasets, analytical outputs, and syntax scripts. Furthermore, parameter selection forces users through deeply nested modal windows.

This platform consolidates workflows into a unified split-screen environment:
1.  **Left Workspace:** Continuous access to the interactive spreadsheet grid, toggleable between Data View and Variable View.
2.  **Contextual Analysis Drawer:** Analyses are configured in a non-modal sliding drawer. Variable assignment lists allow researchers to drag or click variables into Dependent, Independent, or Factor slots while keeping the primary dataset visible.
3.  **Right Workspace (Output Viewer):** Analysis results render sequentially in an interactive output panel. A structured outline navigation tree on the left allows instant scrolling through previous tests, while the main viewer renders publication-ready tables formatted according to American Psychological Association (APA) 7th Edition guidelines. Tables support instant one-click copying to Markdown, LaTeX, and formatted HTML.

---

## 4. Repository Structure

The codebase is organized as a monorepo that separates the mathematical engine, client application, and desktop packaging layers.

```text
spss-alternative/
├── engine/                       # Core Rust computational crate
│   ├── Cargo.toml                # Dependencies: wasm-bindgen, statrs, nalgebra
│   └── src/
│       ├── lib.rs                # WASM entrypoints and module export definitions
│       ├── descriptives.rs       # Central tendency, dispersion, percentiles
│       ├── parametric.rs         # Student's t-test, Welch's t-test, Paired t-test
│       ├── non_parametric.rs     # Mann-Whitney U, Wilcoxon Signed-Rank
│       ├── anova.rs              # One-Way ANOVA, Sums of Squares, Tukey HSD
│       ├── regression.rs         # Simple and Multiple OLS Linear Regression
│       ├── correlation.rs        # Pearson and Spearman correlation matrices
│       └── sav_parser/           # SPSS .sav and .zsav binary file format parser
├── web/                          # Vue 3 + TypeScript web client
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts            # Configured with vite-plugin-wasm & top-level-await
│   ├── src/
│   │   ├── components/
│   │   │   ├── grid/             # Virtualized canvas Data View and Variable View
│   │   │   ├── analysis/         # Side-drawer parameter selectors
│   │   │   └── output/           # APA 7th Edition table renderer and navigation tree
│   │   ├── stores/               # Pinia stores for dataset, metadata, and results
│   │   ├── workers/
│   │   │   └── stats.worker.ts   # Web Worker hosting the WASM engine via Comlink
│   │   └── wasm/                 # Generated WASM binary and JS glue code
├── src-tauri/                    # Tauri desktop shell configuration
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/main.rs               # Native window lifecycle and file system bindings
├── docker/                       # Self-hosted container deployment manifests
│   ├── Dockerfile
│   └── nginx.conf                # Static server with Cross-Origin Isolation headers
└── tests/
    └── r_benchmarks/             # Automated validation test suite verifying against R
```

---

## 5. Getting Started and Development Workflow

Follow this sequence to configure your development environment, compile the mathematical engine to WebAssembly, and boot the reactive frontend.

### 5.1 System Prerequisites

Ensure the following runtimes and compilers are installed locally:
*   **Node.js:** Version 18.0.0 LTS or higher, with `npm` or `pnpm`.
*   **Rust Toolchain:** Stable channel via `rustup`.
*   **WASM Target:** Add the WebAssembly compilation target to Rust:
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
*   **wasm-pack:** Tool for compiling Rust crates to WebAssembly:
    ```bash
    cargo install wasm-pack
    ```
*   **Tauri CLI (Optional for web-only development):**
    ```bash
    cargo install tauri-cli
    ```

### 5.2 Initial Setup and WASM Compilation

1.  Clone the repository and enter the project directory:
    ```bash
    git clone <repository-url>
    cd spss-alternative
    ```

2.  Compile the Rust statistical engine into WebAssembly bindings directed to the web application:
    ```bash
    cd engine
    wasm-pack build --target web --out-dir ../web/src/wasm
    cd ..
    ```

3.  Install web client dependencies:
    ```bash
    cd web
    npm install
    ```

4.  Launch the local development server:
    ```bash
    npm run dev
    ```
    Access the local environment at the default local Vite port.

### 5.3 Desktop Application Development

To execute the application within the native Tauri desktop shell during active development:

```bash
cd web
npm run tauri dev
```

---

## 6. Self-Hosted and Container Deployment

For institutional environments requiring centralized self-hosted deployment, a multi-stage Docker build produces a hardened static web distribution. 

Because modern WebAssembly multithreading and high-performance shared memory buffers depend on `SharedArrayBuffer`, the production HTTP server must serve explicit security headers enforcing Cross-Origin Opener Policy and Cross-Origin Embedder Policy.

### 6.1 Required HTTP Security Headers

The serving layer (such as Nginx or Caddy) must supply the following response headers on all routes:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

### 6.2 Docker Compose Execution

Deploy the pre-configured container stack using Docker Compose:

```bash
docker compose up -d --build
```

The container packages an optimized build of the Vue 3 application alongside compiled WebAssembly assets, served via an Nginx alpine image configured with required cross-origin isolation headers.

---

## 7. Statistical Capabilities and Validation Rigor

Statistical packages must guarantee rigorous mathematical accuracy. Empirical researchers cannot adopt software whose outputs diverge from accepted scientific standards.

### 7.1 Implemented Analytical Routines

The engine delivers verified statistical algorithms across the most common procedural categories:

1.  **Descriptive Statistics:** Sample size ($N$), arithmetic mean, sample variance, standard deviation, minimum, maximum, median, skewness, kurtosis, and custom percentiles.
2.  **Parametric Hypothesis Testing:**
    *   Independent Samples T-Test (reporting standard Student's $t$ with pooled variance and Welch's $t$ for unequal variances).
    *   Paired Samples T-Test for repeated measures.
    *   Calculation of degrees of freedom, two-tailed $p$-values, mean differences, standard error of the difference, and effect sizes (Cohen's $d$ and Hedges' $g$).
3.  **Non-Parametric Hypothesis Testing:**
    *   Mann-Whitney $U$ test with asymptotic and exact $p$-value estimations.
    *   Wilcoxon Signed-Rank test for paired ranks.
4.  **Analysis of Variance (ANOVA):**
    *   One-Way ANOVA computing Sums of Squares (Between and Within), Mean Squares, Snedecor's $F$-ratio, and $p$-values.
    *   Post-hoc pairwise comparisons via Tukey's Honestly Significant Difference (HSD) test.
5.  **Correlation and Linear Regression:**
    *   Bivariate Pearson product-moment and Spearman rank correlation matrices with significance flags.
    *   Simple and Multiple Ordinary Least Squares (OLS) Linear Regression producing unstandardized coefficients ($B$), standard errors, standardized coefficients ($\beta$), $t$-statistics, $p$-values, model $R^2$, adjusted $R^2$, residual standard error, and omnibus $F$-test.

### 7.2 Numerical Validation Methodology

To ensure computational fidelity, the test harness enforces automated continuous integration validation against the gold-standard reference platform, R:

*   **Tolerance Threshold:** All computed test statistics, degrees of freedom, and significance probabilities must match reference outputs from R packages to a minimum of 4 decimal places.
*   **Edge Case Handling:** Validation suites explicitly evaluate unbalanced factorial designs, zero-variance columns, small sample sizes ($N < 5$), and datasets with missing value patterns.

---

## 8. Implementation Roadmap

The project follows a structured implementation roadmap designed for both full-time (10 weeks, 2.5 to 4 months) and part-time (10 to 15 hours per week, 6 to 9 months) development schedules:

*   **Week 1 (Scaffolding & WASM Pipeline):** Monorepo structure, Vue 3, Vite, Tailwind CSS, Pinia, Rust `wasm-pack` compilation pipeline, `comlink` Web Worker communication, and basic descriptive statistics.
*   **Weeks 2-3 (Data Grid & State Model):** Virtualized canvas grid integration, Pinia Dual View state store (Data View records matrix and Variable View metadata table), and CSV/TSV drag-and-drop import/export.
*   **Weeks 4-5 (Parametric & Non-Parametric Tests):** Rust implementations of Student's and Welch's Independent T-Test, Paired T-Test, Mann-Whitney U, and Wilcoxon Signed-Rank; automated reference testing against R benchmarks.
*   **Weeks 6-7 (ANOVA & Linear Models):** One-Way ANOVA with Sums of Squares, F-distribution, and Tukey HSD; Pearson and Spearman correlation matrices; OLS Multiple Linear Regression.
*   **Week 8 (SPSS-Style Side Drawer & APA Output):** Collapsible analysis parameter drawer with variable assignment lists; APA 7th Edition Output Viewer with outline tree; clipboard export to Markdown, LaTeX, and HTML.
*   **Week 9 (.sav File Support & Self-Hosted Packaging):** Binary `.sav` and `.zsav` file format parser integration; Dockerfile and Docker Compose manifests with cross-origin isolation headers.
*   **Week 10 (Tauri Desktop Shell & Release):** Tauri integration, native OS file dialogs, cross-platform release builds for Windows, macOS, and Linux, and public open-source v1.0 release tagging.
