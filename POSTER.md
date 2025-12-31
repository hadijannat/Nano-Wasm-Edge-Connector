# Nano-Wasm Edge Connector: Academic Poster

## For: A0 Format (841mm × 1189mm) Academic Poster

---

## TITLE BLOCK

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│    NANO-WASM EDGE CONNECTOR                                                  │
│    Lightweight Policy Enforcement for Industrial Data Spaces                 │
│                                                                              │
│    WebAssembly-based Access Control for Edge Devices                         │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## SECTION 1: MOTIVATION (Top Left)

### The Challenge

- Industrial Data Spaces (IDS) require policy enforcement at every data exchange
- Traditional Java-based connectors: **500MB+ memory footprint**
- Edge devices (PLCs, gateways): **typically <64MB RAM available**
- Gap: No production-ready solution for edge policy enforcement

### Research Question

> *Can WebAssembly enable sub-10MB policy enforcement for Industrial Data Space connectors?*

---

## SECTION 2: ARCHITECTURE (Top Right)

### System Overview

```
┌─────────────────────────────────────────────────┐
│              HTTP API Layer                     │
│  ┌──────────────────────────────────────────┐   │
│  │ /health  /evaluate  /reload  /metrics    │   │
│  └──────────────────────────────────────────┘   │
│                      │                          │
│  ┌───────────────────▼──────────────────────┐   │
│  │         WASMTIME RUNTIME (v27)           │   │
│  │  • Fuel Metering (1M ops)                │   │
│  │  • Per-request Store Isolation           │   │
│  │  • 64KB Stack Limit                      │   │
│  └───────────────────┬──────────────────────┘   │
│                      │                          │
│  ┌───────────────────▼──────────────────────┐   │
│  │       POLICY MODULE (772 bytes WAT)      │   │
│  │  • Pattern-based Access Control          │   │
│  │  • Hot-swappable at Runtime              │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Technology | Justification |
|-----------|------------|---------------|
| Runtime | Rust + Wasmtime 27 | Memory safety, AOT compilation |
| HTTP | Axum | Async, minimal overhead |
| Policy | Hand-written WAT | Maximum size optimization |
| Watcher | notify-debouncer-mini | Hot-reload with debouncing |

---

## SECTION 3: POLICY MODEL (Middle Left)

### Access Control Rules (Implemented in WAT)

```
RULE 1: blocked:true      → DENY   (explicit block)
RULE 2: role = "admin"    → ALLOW  (superuser)
RULE 3: role = "operator" → DENY if resource contains "secret"
                          → ALLOW otherwise
RULE 4: role = "viewer"   → DENY if action contains "write"
                          → ALLOW otherwise
RULE 5: default           → ALLOW  (permissive edge policy)
```

### Pattern Matching Implementation

- Byte-level comparison in WAT `$contains` function
- No regex overhead
- Linear time complexity: O(n × m)

---

## SECTION 4: RESULTS (Middle Right)

### Binary Size Comparison

| Component | Nano-Wasm | Traditional IDS |
|-----------|-----------|-----------------|
| **Binary** | 2.5 MB | 50-100 MB |
| **Policy** | 772 bytes | 10-100 KB |
| **Runtime Memory** | ~10 MB | 500+ MB |

### Performance Metrics

| Metric | Value |
|--------|-------|
| Policy evaluation latency | <5 ms |
| Fuel limit per request | 1,000,000 ops |
| Memory isolation | Per-request Store |
| Hot-reload latency | <100 ms |

### Test Matrix (8/8 Passed)

| Test Case | Expected | Result |
|-----------|----------|--------|
| Admin access | ALLOW | ✅ |
| Blocked request | DENY | ✅ |
| Operator normal | ALLOW | ✅ |
| Operator + secret | DENY | ✅ |
| Viewer read | ALLOW | ✅ |
| Viewer + write | DENY | ✅ |
| Default guest | ALLOW | ✅ |

---

## SECTION 5: TECHNICAL CHALLENGES (Bottom Left)

### Challenge 1: Wasm Memory Access

**Problem:** Rust `no_std` for `wasm32-unknown-unknown` lacked proper memory initialization.

**Solution:** Hand-written WAT with explicit memory export and data segments.

### Challenge 2: Epoch Interruption Traps

**Problem:** Wasmtime's `epoch_interruption` caused unexpected traps during execution.

**Solution:** Disabled epoch interruption; rely on fuel metering for DoS protection.

### Challenge 3: Host Function Memory Access

**Problem:** Log function needed access to Wasm linear memory from host.

**Solution:** Use `Caller::get_export("memory")` to access instance memory dynamically.

---

## SECTION 6: FUTURE WORK (Bottom Middle)

1. **Integration with Eclipse Dataspace Connector (EDC)**
   - Implement as EDC policy extension
   - Benchmark against default Java policy engine

2. **Policy Language**
   - DSL-to-WAT compiler for non-expert policy authoring
   - Support for ODRL subset

3. **Edge Deployment**
   - Raspberry Pi 4 validation
   - ARM64 cross-compilation
   - Container-less deployment (static binary)

4. **Formal Verification**
   - WebAssembly specification compliance
   - Policy correctness proofs

---

## SECTION 7: CONCLUSION (Bottom Right)

### Key Contributions

1. **Proof of concept** for sub-10MB IDS policy enforcement
2. **Documented workarounds** for Wasmtime 27 edge deployment
3. **Open-source implementation** with comprehensive testing guide

### Metrics Summary

```
┌────────────────────────────────────┐
│  ✅ Host Binary:     2.5 MB        │
│  ✅ Policy Module:   772 bytes     │
│  ⚠️ Runtime Memory:  ~10 MB       │
│  ✅ All Tests:       8/8 passed    │
└────────────────────────────────────┘
```

### Availability

- **Source Code:** GitHub (link)
- **Testing Guide:** TESTING.md (462 lines)
- **License:** Apache 2.0

---

## QR CODE BLOCK (Bottom Right Corner)

```
┌─────────────┐
│             │
│   [QR CODE] │
│             │
│  Scan for   │
│  Source     │
└─────────────┘
```

---

## VISUAL ASSETS TO INCLUDE

1. **Architecture diagram** (generated) - `docs/architecture.png`
2. **Binary size comparison bar chart**
3. **Memory footprint line graph** (over 100 requests)
4. **Test matrix table with checkmarks**
5. **QR code** to GitHub repository

---

## COLOR SCHEME

| Element | Color | Hex |
|---------|-------|-----|
| Primary | Dark Blue | #1E3A5F |
| Accent | Orange | #FF6B35 |
| Success | Green | #28A745 |
| Warning | Yellow | #FFC107 |
| Background | White | #FFFFFF |

---

## TYPOGRAPHY

- **Title:** Montserrat Bold, 72pt
- **Section Headers:** Montserrat SemiBold, 36pt
- **Body:** Inter Regular, 18pt
- **Code:** JetBrains Mono, 14pt
