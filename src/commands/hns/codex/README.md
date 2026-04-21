# hara-abi-endec — ABI Binary Codec

A compact binary encoder/decoder for Ethereum ABI definitions.
Converts standard ABI JSON (bare arrays or Hardhat/Foundry artifact objects) into a space-efficient binary blob, and back.

---

## Table of Contents

1. [Why a binary encoding?](#1-why-a-binary-encoding)
2. [Binary Format Grammar](#2-binary-format-grammar)
3. [Type Tag Reference](#3-type-tag-reference)
4. [Entry Kind Reference](#4-entry-kind-reference)
5. [Encoding Simulation — step by step](#5-encoding-simulation--step-by-step)
   - [5.1 The input ABI](#51-the-input-abi)
   - [5.2 Encoding a Function](#52-encoding-a-function)
   - [5.3 Encoding an Event](#53-encoding-an-event)
   - [5.4 Encoding an Error](#54-encoding-an-error)
   - [5.5 Assembling the blob](#55-assembling-the-blob)
   - [5.6 Full annotated hex dump](#56-full-annotated-hex-dump)
6. [Complex Type Encoding](#6-complex-type-encoding)
   - [6.1 Array](#61-array)
   - [6.2 Tuple (struct)](#62-tuple-struct)
   - [6.3 Nested tuple](#63-nested-tuple)
   - [6.4 Array of tuples](#64-array-of-tuples)
7. [Decoding walkthrough](#7-decoding-walkthrough)
8. [CLI Usage](#8-cli-usage)
9. [JSON Input Formats](#9-json-input-formats)

---

## 1. Why a binary encoding?

Ethereum ABI JSON is human-readable but verbose — a typical contract ABI may weigh 10–50 KB.
The binary format collapses redundant field names, replaces type strings with 1-byte tags, and prefixes each entry with a compact length, making blobs suitable for on-chain storage via EIP-205 (`setAbi`) with `contentType = 8` (binary).

---

## 2. Binary Format Grammar

```
blob       := entry_count:u8  entry*

entry      := kind:u8  payload_len:u16be  payload:bytes

kind       := 0x01   ; function
           |  0x02   ; event
           |  0x03   ; error

; ── function payload ────────────────────────────────────────────────
fn_payload := selector:4bytes  name_len:u8  name:ascii
              input_count:u8   type*
              output_count:u8  type*

; ── event payload ─────────────────────────────────────────────────
; topic0 is NOT stored — reconstruct with keccak256(event_sig) after decoding.
ev_payload := name_len:u8  name:ascii
              param_count:u8   ev_param*

ev_param   := indexed:u8  type          ; indexed: 0x00 or 0x01

; ── error payload ─────────────────────────────────────────────────
er_payload := selector:4bytes  name_len:u8  name:ascii
              param_count:u8   type*

; ascii := raw ASCII bytes (Solidity identifiers are always ASCII)

; ── recursive type ─────────────────────────────────────────────────
type       := 0x01             ; uint256
           |  0x02             ; address
           |  0x03             ; bool
           |  0x04             ; bytes32
           |  0x05             ; string
           |  0x06             ; bytes
           |  0x07             ; uint8
           |  0x08             ; uint16
           |  0x09             ; uint32
           |  0x0A             ; uint64
           |  0x0B             ; uint128
           |  0x0C  n:u8       ; bytesN where n ∈ 1..4
           |  0x20  type       ; T[]   (array — recurses into inner type)
           |  0x30  count:u8  type*   ; tuple — count fields follow
```

---

## 3. Type Tag Reference

| Tag    | Hex  | Solidity type(s)               |
|--------|------|-------------------------------|
| UINT256 | `0x01` | `uint256`, `int256`          |
| ADDRESS | `0x02` | `address`                    |
| BOOL    | `0x03` | `bool`                       |
| BYTES32 | `0x04` | `bytes32`                    |
| STRING  | `0x05` | `string`                     |
| BYTES   | `0x06` | `bytes`                      |
| UINT8   | `0x07` | `uint8`, `int8`              |
| UINT16  | `0x08` | `uint16`, `int16`            |
| UINT32  | `0x09` | `uint32`, `int32`            |
| UINT64  | `0x0A` | `uint64`, `int64`            |
| UINT128 | `0x0B` | `uint128`, `int128`          |
| BYTES_N | `0x0C` | `bytes1`…`bytes4` (followed by size `u8`) |
| ARRAY   | `0x20` | `T[]` / `T[N]` — followed by inner type tag |
| TUPLE   | `0x30` | `(T1,T2,…)` — followed by `u8` field count and field type tags |

---

## 4. Entry Kind Reference

| Kind     | Hex    | Hash stored?           | Notes |
|----------|--------|------------------------|-------|
| Function | `0x01` | ✓ 4-byte selector      | `keccak256(sig)[0..4]` |
| Event    | `0x02` | ✗ none                 | topic0 reconstructable from name + params |
| Error    | `0x03` | ✓ 4-byte selector      | `keccak256(sig)[0..4]` |

**Why events don't store topic0:** The full 32-byte hash is `keccak256(event_signature)`,
and the signature is fully derivable from the stored ASCII name and param types.
Storing topic0 would waste 32 bytes per event — ~33% overhead on a small event payload.

To reconstruct topic0 after decoding, use the exported helpers:
```rust
use encode::{event_sig, topic0};
let sig  = event_sig(&event.name, &event.params); // e.g. "Transfer(address,address,uint256)"
let t0   = topic0(&sig);                          // [u8; 32]
```

**Canonical signature** rules (Solidity ABI spec):
- `functionName(type1,type2,…)` — no spaces
- Tuples expand to `(innerType1,innerType2,…)`
- Arrays expand to `innerType[]`

**Name encoding:** All Solidity identifiers are ASCII. Names are stored as raw ASCII bytes
(no null terminator). Since ASCII ⊂ UTF-8, the bytes round-trip cleanly through
`std::str::from_utf8` / `.as_bytes()`.

---

## 5. Encoding Simulation — step by step

### 5.1 The input ABI

We'll encode this small contract:

```solidity
function transfer(address to, uint256 amount) external returns (bool);

event Transfer(address indexed from, address indexed to, uint256 value);

error InsufficientBalance(address account, uint256 needed);
```

Total entries = **3** (1 function + 1 event + 1 error).

---

### 5.2 Encoding a Function

**Step A — Build the canonical signature**

```
transfer(address,uint256)
```

**Step B — Compute the 4-byte selector**

```
keccak256("transfer(address,uint256)") =
  a9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b

selector = a9 05 9c bb   (first 4 bytes)
```

**Step C — Encode name (ASCII)**

```
"transfer" → ASCII bytes: 74 72 61 6e 73 66 65 72
name_len   = 08
```

**Step D — Encode inputs**

| Input  | Canonical | Tag  |
|--------|-----------|------|
| address | `address` | `02` |
| uint256 | `uint256` | `01` |

```
input_count = 02
types       = 02 01
```

**Step E — Encode outputs**

| Output | Canonical | Tag  |
|--------|-----------|------|
| bool   | `bool`    | `03` |

```
output_count = 01
types        = 03
```

**Step F — Assemble the function payload**

```
Offset  Bytes              Meaning
0       a9 05 9c bb        selector
4       08                 name_len = 8
5       74 72 61 6e        name[0..3] "tran"
9       73 66 65 72        name[4..7] "sfer"
13      02                 input_count = 2
14      02                 input[0] = ADDRESS
15      01                 input[1] = UINT256
16      01                 output_count = 1
17      03                 output[0] = BOOL

Payload size = 18 bytes  (0x0012)
```

---

### 5.3 Encoding an Event

> **Design decision:** `topic0` is **not stored** in the binary blob.
> The 32-byte hash is `keccak256(canonical_signature)` and the signature
> is fully reconstructable from the stored name + param types.
> Omitting it saves **32 bytes per event** with zero information loss.


**Step A — Encode name (ASCII)**

```
"Transfer" → ASCII bytes: 54 72 61 6e 73 66 65 72
name_len   = 08
```

**Step B — Encode params (with indexed flag)**

| Param   | Indexed | Type    | Bytes      |
|---------|---------|---------|------------|
| from    | ✓       | address | `01 02`    |
| to      | ✓       | address | `01 02`    |
| value   | ✗       | uint256 | `00 01`    |

**Step C — Assemble the event payload**

```
Offset  Bytes                      Meaning
0       08                         name_len = 8
1       54 72 61 6e 73 66 65 72   name = "Transfer"  (ASCII)
9       03                         param_count = 3
10      01 02                      param[0]: indexed=YES  ADDRESS
12      01 02                      param[1]: indexed=YES  ADDRESS
14      00 01                      param[2]: indexed=NO   UINT256

Payload size = 16 bytes  (0x0010)
Saving      = 32 bytes  per event
```

---

### 5.4 Encoding an Error

**Step A — Canonical signature**

```
InsufficientBalance(address,uint256)
```

**Step B — 4-byte selector**

```
keccak256("InsufficientBalance(address,uint256)") =
  cf479181...

selector = cf 47 91 81
```

**Step C — Name**

```
"InsufficientBalance" → 19 bytes
49 6e 73 75 66 66 69 63 69 65 6e 74 42 61 6c 61 6e 63 65
name_len = 19  (hex: 0x13)
```

**Step D — Params**

```
param_count = 02
param[0]    = 02  (ADDRESS)
param[1]    = 01  (UINT256)
```

**Step E — Assemble the error payload**

```
Offset  Bytes                                              Meaning
0       cf 47 91 81                                       selector
4       13                                                name_len = 19
5       49 6e 73 75 66 66 69 63 69 65 6e 74              "Insufficient"
17      42 61 6c 61 6e 63 65                              "Balance"
24      02                                                param_count = 2
25      02                                                param[0] = ADDRESS
26      01                                                param[1] = UINT256

Payload size = 27 bytes  (0x001b)
```

---

### 5.5 Assembling the blob

Each entry is wrapped with: `kind(1) + payload_len(2 BE) + payload`.

```
Byte 0:  03          ← total entry count (1 fn + 1 event + 1 error)

Entry 1 — Function
  Byte  1: 01         ← ENTRY_FUNCTION
  Bytes 2-3: 00 12    ← payload_len = 18
  Bytes 4-21: [function payload — 18 bytes]

Entry 2 — Event
  Byte 22: 02         ← ENTRY_EVENT
  Bytes 23-24: 00 10  ← payload_len = 16  (no topic0 stored)
  Bytes 25-40: [event payload — 16 bytes]

Entry 3 — Error
  Byte 41: 03         ← ENTRY_ERROR
  Bytes 42-43: 00 1b  ← payload_len = 27
  Bytes 44-70: [error payload — 27 bytes]

Total blob size = 1 + (3×3) + 18 + 16 + 27 = 71 bytes
                                         ↑ was 103 with topic0 stored per event
```

---

### 5.6 Full annotated hex dump

```
Offset  Hex bytes                                 Annotation
──────  ────────────────────────────────────────  ─────────────────────────────
000000  03                                        entry_count = 3

        ── Entry 0: Function ──────────────────────────────────────────────────
000001  01                                        kind = FUNCTION
000002  00 12                                     payload_len = 18
          ── payload ─────────────────────────────────────────────────────────
000004  a9 05 9c bb                               selector  keccak256("transfer(address,uint256)")[0..4]
000008  08                                        name_len = 8
000009  74 72 61 6e 73 66 65 72                   name = "transfer"  (ASCII)
000011  02                                        input_count = 2
000012  02                                        input[0] = ADDRESS  (0x02)
000013  01                                        input[1] = UINT256  (0x01)
000014  01                                        output_count = 1
000015  03                                        output[0] = BOOL    (0x03)

        ── Entry 1: Event ─────────────────────────────────────────────────────
000016  02                                        kind = EVENT
000017  00 10                                     payload_len = 16  ← no topic0
          ── payload ─────────────────────────────────────────────────────────
000019  08                                        name_len = 8
000020  54 72 61 6e 73 66 65 72                   name = "Transfer"  (ASCII)
000028  03                                        param_count = 3
000029  01 02                                     param[0]: indexed=YES  ADDRESS
000031  01 02                                     param[1]: indexed=YES  ADDRESS
000033  00 01                                     param[2]: indexed=NO   UINT256

        ── Entry 2: Error ─────────────────────────────────────────────────────
000035  03                                        kind = ERROR
000036  00 1b                                     payload_len = 27
          ── payload ─────────────────────────────────────────────────────────
000038  cf 47 91 81                               selector  keccak256("InsufficientBalance(address,uint256)")[0..4]
000042  13                                        name_len = 19
000043  49 6e 73 75 66 66 69 63 69 65 6e 74      name[0..11]  "Insufficient"  (ASCII)
000055  42 61 6c 61 6e 63 65                      name[12..18] "Balance"
000062  02                                        param_count = 2
000063  02                                        param[0] = ADDRESS
000064  01                                        param[1] = UINT256

        Total: 71 bytes  (was 103 with topic0 stored; -32 bytes per event)
```

---

## 6. Complex Type Encoding

### 6.1 Array

`uint256[]` — an unbounded array of uint256.

```
0x20  0x01
  │     └── inner type = UINT256
  └── MOD_ARRAY tag
```

`address[][]` — 2-D array:

```
0x20  0x20  0x02
  │     │     └── innermost = ADDRESS
  │     └── second MOD_ARRAY
  └── first MOD_ARRAY
```

### 6.2 Tuple (struct)

```solidity
struct Transfer { address to; uint256 amount; }
```

Canonical form: `(address,uint256)`

```
0x30  0x02  0x02  0x01
  │     │     │     └── field[1] = UINT256
  │     │     └── field[0] = ADDRESS
  │     └── field_count = 2
  └── MOD_TUPLE tag
```

### 6.3 Nested tuple

```solidity
struct Inner { bool flag; bytes32 id; }
struct Outer { address owner; Inner meta; }
```

Canonical form: `(address,(bool,bytes32))`

```
0x30  0x02  0x02  0x30  0x02  0x03  0x04
  │     │     │     │     │     │     └── field[1] of Inner = BYTES32
  │     │     │     │     │     └── field[0] of Inner = BOOL
  │     │     │     │     └── Inner.field_count = 2
  │     │     │     └── Inner is MOD_TUPLE
  │     │     └── field[0] of Outer = ADDRESS
  │     └── Outer.field_count = 2
  └── Outer is MOD_TUPLE
```

Total type bytes: **7**

### 6.4 Array of tuples

```solidity
struct Hop { address token; uint256 min; }
Hop[] hops
```

Canonical form: `(address,uint256)[]`

```
0x20  0x30  0x02  0x02  0x01
  │     │     │     │     └── field[1] = UINT256
  │     │     │     └── field[0] = ADDRESS
  │     │     └── field_count = 2
  │     └── inner is MOD_TUPLE
  └── MOD_ARRAY (outer wrapper)
```

Total type bytes: **5**

---

## 7. Decoding walkthrough

The decoder streams left-to-right through the blob using a `Cursor`:

```
1. Read 1 byte  → entry_count
2. Loop entry_count times:
   a. Read 1 byte  → kind (0x01 / 0x02 / 0x03)
   b. Read 2 bytes → payload_len (u16 big-endian)
   c. Read payload_len bytes → sub-slice
   d. Dispatch to decode_function / decode_event / decode_error
      according to kind
3. Inside each decoder:
   Function / Error → read 4-byte selector, ASCII name, then types
   Event           → read ASCII name directly (no topic0 prefix), then params
   Types (recursive):
     - 0x01–0x06  → primitive, done
     - 0x20       → recurse into one more inner type
     - 0x30       → read field_count u8, recurse field_count times
4. To reconstruct topic0 from a decoded event:
   sig = event_sig(&event.name, &event.params)  // "Transfer(address,address,uint256)"
   t0  = topic0(&sig)                           // [u8; 32]
5. Any read past the end of the buffer → Err("unexpected end of data")
6. Any unrecognized tag             → Err("unknown type tag")
```

This means the decoder **validates structure** completely and returns a typed `AbiBlob`
— it never produces partial results.

---

## 8. CLI Usage

```powershell
# Encode a single ABI JSON
cargo run -- .\Token.json
# → writes Token.abi.bin in the same directory

# Encode all JSON files in a folder (recursive)
cargo run -- .\abis\

# Inspect a binary blob — hex dump + decoded JSON
cargo run -- inspect .\Token.abi.bin
```

### inspect output

```
══════════════════════════════════════════════════════════
  File    : Token.abi.bin
  Size    : 71 bytes
══════════════════════════════════════════════════════════

── Hex Dump ───────────────────────────────────────────────
  00000000  03 01 00 12 a9 05 9c bb  08 74 72 61 6e 73 66 65  │ .........transfer │
  00000010  72 02 02 01 01 03 02 00  10 08 54 72 61 6e 73 66  │ r.........Transf  │
  00000020  65 72 03 01 02 01 02 00  01 03 00 1b cf 47 91 81  │ er...........G..  │
  ...

── Decoded Entries ────────────────────────────────────────
  Functions : 1
  Events    : 1
  Errors    : 1

── Decoded JSON ───────────────────────────────────────────
[
  {
    "kind": "function",
    "name": "transfer",
    "inputs": [{"type": "address"}, {"type": "uint256"}],
    "outputs": [{"type": "bool"}]
  },
  ...
]
```

---

## 9. JSON Input Formats

The encoder accepts two common JSON shapes:

### Bare array (standard ABI output)
```json
[
  { "type": "function", "name": "transfer", "inputs": [...], "outputs": [...] },
  { "type": "event",    "name": "Transfer", "inputs": [...] },
  { "type": "error",    "name": "InsufficientBalance", "inputs": [...] }
]
```

### Artifact object (Hardhat / Foundry)
```json
{
  "abi": [ ... ],
  "contract_address": "0x...",
  "args": [...]
}
```

The parser automatically detects the root shape.  
If the root is an object, it extracts the `"abi"` field.  
If no `"abi"` field is present, an error is returned.

### Skipped entry types

| Type          | Reason skipped                    |
|---------------|-----------------------------------|
| `constructor` | No stable name to hash            |
| `receive`     | No parameters to encode           |
| `fallback`    | No parameters to encode           |

---

## Appendix — Size comparison

| Format       | Example size (ERC-20) | Notes                                |
|---|---|---|
| ABI JSON     | ~4 400 bytes          | Human-readable, verbose field names  |
| ABI binary   | ~230 bytes            | ~19× smaller (no topic0 per event)   |

### Design choices summary

| Choice              | Decision         | Rationale                                           |
|---------------------|------------------|-----------------------------------------------------|
| Name encoding       | Raw ASCII bytes  | Solidity identifiers are always ASCII; no overhead  |
| Event topic0        | Not stored       | Reconstructable — saves 32 bytes per event          |
| Function selector   | Stored (4 bytes) | Not trivially reconstructible without type context  |
| Error selector      | Stored (4 bytes) | Same reason as function selector                    |
| Payload length      | u16 big-endian   | Supports up to 64 KB per entry; simple to parse     |
| Type tags           | 1 byte each      | 8 distinct values needed; u8 is sufficient          |

The binary is suited for on-chain storage via EIP-205 with `contentType = 8`.
