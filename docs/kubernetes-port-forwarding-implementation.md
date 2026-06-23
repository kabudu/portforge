# Kubernetes Port-Forwarding Awareness Implementation Plan

## Summary

Implement Phase 3 Kubernetes pod port-forwarding awareness by detecting local
listeners created by `kubectl port-forward` and enriching PortForge entries with
the forwarded Kubernetes target.

This is a local process/command-line enrichment feature. It must not call the
Kubernetes API, read kubeconfig, contact clusters, or run `kubectl`. The first
production slice should be deterministic, fast, cross-platform, and safe to run
on every scan.

Roadmap source: `ROADMAP.md`, Phase 3, "Kubernetes pod port-forwarding
awareness".

## Goals

- Show when a local listening port is backed by a Kubernetes port-forward.
- Parse enough command-line metadata to identify the Kubernetes target.
- Surface Kubernetes metadata consistently in JSON, CSV, table, inspect, TUI,
  and web dashboard views.
- Preserve the current scan performance profile by doing only in-memory parsing
  per listener.
- Keep detection best-effort: unknown or unusual commands must not fail a scan.

## Non-Goals

- Managing or killing remote Kubernetes workloads.
- Opening, closing, or restarting port-forward sessions.
- Watching the Kubernetes API for pod/service state.
- Reading kubeconfig, current context, or namespace from disk.
- Supporting every shell quoting edge case beyond the command data exposed by
  `sysinfo`.
- Marking the roadmap item complete before display surfaces, docs, and tests are
  implemented.

## Acceptance Criteria

- `kubectl port-forward` listeners are detected when the process name or command
  line identifies `kubectl` and the command contains the `port-forward`
  subcommand.
- The detector parses common target forms:
  - `pod/api-123`
  - `pods/api-123`
  - `svc/api`
  - `service/api`
  - `deployment/web`
  - `deploy/web`
- The detector parses namespace flags:
  - `-n dev`
  - `--namespace dev`
  - `--namespace=dev`
- The detector parses context flags:
  - `--context staging`
  - `--context=staging`
- The detector parses address flags when present:
  - `--address 127.0.0.1`
  - `--address=0.0.0.0`
- The detector handles port specs:
  - `8080:80`
  - `8080`
  - `:8080`
  - multiple port specs, selecting the one matching the listener port when
    possible.
- `PortEntry` exposes optional Kubernetes metadata in serialized output without
  breaking deserialization of older fixtures or JSON consumers.
- CLI table and CSV output include Kubernetes information.
- `portforge inspect <port>` includes a Kubernetes section when metadata is
  present.
- TUI table or detail view includes Kubernetes information without making the
  existing table unusable on common terminal widths.
- Web dashboard table or detail modal includes Kubernetes information with HTML
  escaping.
- README documents the feature and output meaning.
- Roadmap is updated only after the feature is implemented and validated.

## Proposed Data Model

Add a new optional field to `PortEntry`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub kubernetes: Option<KubernetesInfo>,
```

Add a new model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KubernetesInfo {
    pub tool: String,
    pub resource_kind: String,
    pub resource_name: String,
    pub namespace: Option<String>,
    pub context: Option<String>,
    pub local_port: u16,
    pub remote_port: Option<u16>,
    pub bind_address: Option<String>,
}
```

Add display helpers on `PortEntry` or `KubernetesInfo`:

- Compact table value: `svc/api:8080->80` or `pod/api:8080`
- Detail value:
  - Resource
  - Namespace
  - Context
  - Local port
  - Remote port
  - Bind address

The model should avoid storing the full command line because `PortEntry.command`
already carries that data and command lines may include sensitive arguments.

## Module Boundary

Add `src/kubernetes.rs` with pure parsing and detection helpers:

```rust
pub fn detect_port_forward(
    listener_port: u16,
    process_name: &str,
    command: &str,
) -> Option<KubernetesInfo>
```

Ownership:

- `kubernetes.rs` owns command detection and parsing.
- `scanner.rs` owns invoking the detector and attaching metadata to `PortEntry`.
- `models.rs` owns serialized model shape and display helpers.
- `export.rs`, `tui/ui.rs`, and `web/handlers.rs` own their respective
  presentation surfaces.

Do not put Kubernetes parsing inside `scanner.rs`; the scanner should remain an
orchestrator for enrichment sources.

## Parsing Strategy

Use a small local parser over the command string already available from
`sysinfo`. Start with whitespace tokenization because PortForge currently joins
process args with spaces and does not preserve original shell quoting.

Detection rules:

- Match tools: `kubectl` first. Optionally allow `oc` only if tests and docs
  explicitly cover it.
- Require a `port-forward` token.
- Accept global flags before or after `port-forward`.
- Find the first non-flag token after `port-forward` that looks like a resource
  target.
- Parse resource target as `kind/name`.
- Normalize common kind aliases:
  - `po`, `pod`, `pods` -> `pod`
  - `svc`, `service`, `services` -> `service`
  - `deploy`, `deployment`, `deployments` -> `deployment`
- Collect subsequent port-spec tokens until another flag begins.
- Select a port spec:
  - Prefer a spec whose explicit local port equals the listener port.
  - If a spec has no explicit local port, use the listener port as the local
    port and parse the token as the remote port.
  - If no spec can be associated with the listener port, return `None`.

Invalid or ambiguous command lines return `None`. They must not produce warnings
unless debugging is enabled, because odd process command lines are normal.

## Failure Modes And Handling

- Missing process metadata: return `None`; scan continues.
- Unsupported target form: return `None`; scan continues.
- Multiple port specs with no listener match: return `None`; avoid showing a
  misleading association.
- Non-UTF or lossy command data: use the existing lossy process-name/command
  strings; avoid panics.
- HTML-sensitive resource names: escape in web output.
- Very long resource names: rely on existing table truncation/layout behavior;
  do not allocate unbounded buffers.

## Performance And Scalability

Expected work is O(number of command tokens) per listener. There is no I/O, no
network access, and no extra process spawning.

Performance constraints:

- No cluster calls.
- No kubeconfig reads.
- No filesystem probes.
- No async tasks.
- No global mutable state.
- Parser tests should include long irrelevant commands to confirm bounded,
  linear behavior.

## Compatibility

- JSON output gains a new optional field. Use serde defaults so older serialized
  fixtures remain readable.
- CSV output gains Kubernetes columns. This is a public output shape change and
  must be documented in README and CHANGELOG when implemented.
- Existing TUI sort fields should continue working. Do not add Kubernetes sort
  until there is a clear user need.
- Existing tunnel detection stays separate. A `kubectl port-forward` is not a
  tunnel in the existing public URL sense and should not populate `TunnelInfo`.

## Implementation Task Checklist

- [x] Add `src/kubernetes.rs` with `detect_port_forward` and parser helpers.
- [x] Add `pub mod kubernetes;` to `src/lib.rs`.
- [x] Add `KubernetesInfo` to `src/models.rs`.
- [x] Add `kubernetes: Option<KubernetesInfo>` to `PortEntry` with serde
      compatibility attributes.
- [x] Add a compact Kubernetes display helper.
- [x] Update every test helper that constructs `PortEntry`.
- [x] Call `kubernetes::detect_port_forward` from `scanner::scan_ports`.
- [x] Keep the scanner failure path best-effort: parser failure returns `None`.
- [x] Add JSON export tests; no existing JSON snapshots required updates.
- [x] Add CSV columns for Kubernetes resource, namespace, context, local port,
      remote port, and bind address.
- [x] Add table output Kubernetes column or compactly append Kubernetes metadata
      to an existing suitable column.
- [x] Add an inspect-mode Kubernetes section.
- [x] Add TUI detail rendering for Kubernetes metadata.
- [x] Decide whether the TUI table gets a Kubernetes column or keeps the table
      width stable and shows Kubernetes only in detail view.
- [x] Add web dashboard Kubernetes rendering with HTML escaping.
- [x] Add README documentation for Kubernetes port-forward awareness.
- [x] Add CHANGELOG `[Unreleased]` entry describing the new feature and CSV
      shape change.
- [x] Update `ROADMAP.md` only after all acceptance criteria pass.

## Parser Unit Test Checklist

- [x] Detects `kubectl port-forward pod/api 8080:80`.
- [x] Detects `kubectl port-forward pods/api 8080:80` and normalizes kind.
- [x] Detects `kubectl port-forward svc/api 3000:3000`.
- [x] Detects `kubectl port-forward service/api 3000`.
- [x] Detects `kubectl port-forward deployment/web :8080`.
- [x] Parses namespace with `-n dev`.
- [x] Parses namespace with `--namespace dev`.
- [x] Parses namespace with `--namespace=dev`.
- [x] Parses context with `--context staging`.
- [x] Parses context with `--context=staging`.
- [x] Parses bind address with `--address 127.0.0.1`.
- [x] Parses bind address with `--address=0.0.0.0`.
- [x] Handles flags before `port-forward`.
- [x] Handles flags after the resource target.
- [x] Selects the matching local port from multiple port specs.
- [x] Returns `None` when no port spec matches the listener port.
- [x] Returns `None` for non-`kubectl` commands.
- [x] Returns `None` for `kubectl` commands without `port-forward`.
- [x] Returns `None` for malformed resource targets.
- [x] Handles long irrelevant commands without excessive allocation or panics.

## Integration And Surface Test Checklist

- [x] Scanner attaches Kubernetes metadata to a synthetic `PortEntry` path where
      possible through focused scanner helper tests.
- [x] `export::to_json` includes `kubernetes` when present.
- [x] `export::to_json` omits or nulls `kubernetes` consistently when absent.
- [x] `export::to_csv` includes Kubernetes columns and escapes resource names.
- [x] `export::to_table` includes the compact Kubernetes value where selected.
- [x] `print_inspection` includes a Kubernetes section only when present.
- [x] TUI detail rendering compiles and is included in full TUI/web validation.
- [x] Web table rendering escapes Kubernetes resource fields; modal rendering
      uses the existing client-side `escapeHtml` helper.
- [x] Existing tests that construct `PortEntry` are updated without weakening
      assertions.

## Validation Commands

Run these before marking the feature complete:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --features web
cargo clippy --features web --all-targets -- -D warnings
```

If the change touches web rendering or web-only code paths, also run:

```bash
cargo build --features web
```

Manual smoke validation:

```bash
kubectl port-forward svc/api 18080:80 -n dev --context staging
portforge inspect 18080
portforge ps --json --all
portforge export --format csv --all
cargo run -- --all
cargo run --features web -- serve --port 9090
```

Expected smoke results:

- `inspect` shows a Kubernetes section for port `18080`.
- JSON includes `kubernetes.resource_kind`, `resource_name`, `namespace`,
  `context`, `local_port`, and `remote_port`.
- CSV includes Kubernetes columns with stable headers.
- TUI detail view shows Kubernetes metadata and remains navigable.
- Web dashboard renders Kubernetes metadata without raw HTML injection.

If `kubectl` or a cluster is unavailable, manual smoke can be replaced by a
temporary local process fixture only if it produces a listener and command line
that matches `kubectl port-forward`. Unit and export tests remain mandatory.

## Review Checklist

- [x] No network or cluster access was introduced.
- [x] No kubeconfig or credential files are read.
- [x] Parser failures cannot fail a scan.
- [x] All new serialized fields are optional or backward-compatible.
- [x] CSV shape change is documented.
- [x] Web output escapes all Kubernetes-derived strings.
- [x] TUI table remains usable at typical terminal widths.
- [x] Existing tunnel behavior remains unchanged.
- [x] Roadmap status matches actual implementation state.

## E2E Validation Run - 2026-06-23

Executed a real local Kubernetes validation on macOS with Docker and `kind`:

```bash
brew install kind
kind create cluster --name portforge-e2e --wait 120s
kubectl --context kind-portforge-e2e create namespace portforge-e2e
kubectl --context kind-portforge-e2e -n portforge-e2e create deployment redis-e2e --image=redis:7-alpine --port=6379
kubectl --context kind-portforge-e2e -n portforge-e2e expose deployment redis-e2e --port=6379 --target-port=6379
kubectl --context kind-portforge-e2e -n portforge-e2e rollout status deployment/redis-e2e --timeout=180s
kubectl --context kind-portforge-e2e -n portforge-e2e port-forward svc/redis-e2e 30080:6379
```

Verified:

- TCP probe to `127.0.0.1:30080` succeeded.
- `portforge --all --json inspect 30080` returned Kubernetes metadata:
  `service/redis-e2e`, namespace `portforge-e2e`, context
  `kind-portforge-e2e`, local port `30080`, remote port `6379`.
- `portforge --all ps --json` included the same Kubernetes metadata for port
  `30080`.
- `portforge --all export --format csv` included:
  `service/redis-e2e,portforge-e2e,kind-portforge-e2e,30080,6379`.
- `portforge serve --port 39090 --bind 127.0.0.1` exposed
  `/api/ports/30080` with the same Kubernetes metadata.
- `/partials/table` rendered the Kubernetes column as
  `portforge-e2e service/redis-e2e:30080-&gt;6379`.

Cleanup:

```bash
kind delete cluster --name portforge-e2e
```

Post-cleanup verification:

- `kind get clusters` reported no clusters.
- No listener remained on `30080`.
- No listener remained on `39090`.

## Release Notes Draft

When implemented, add to `CHANGELOG.md` under `[Unreleased]`:

```markdown
### Added

- **Kubernetes Port-Forward Awareness** - PortForge now detects
  `kubectl port-forward` listeners and surfaces the forwarded resource,
  namespace, context, and port mapping across CLI, TUI, web, JSON, and CSV
  outputs.

### Changed

- **CSV Export** - CSV output now includes Kubernetes metadata columns for
  detected port-forward sessions.
```
