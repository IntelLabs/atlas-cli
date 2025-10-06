# C2PA Transparency Log Service Architecture

## Overview

The C2PA Transparency Log Service is a cryptographically secure, append-only storage system for Content Authenticity Initiative (C2PA) manifests. It implements a verifiable transparency log using Merkle trees, ensuring data integrity and enabling third-party verification of manifest inclusion.
 
## Core Components

### 1. Storage Layer

The service uses MongoDB for persistent storage with two main collections:

#### `manifests` Collection
Stores the actual C2PA manifest data with the following schema:

```javascript
{
  "_id": ObjectId,                    // MongoDB document ID
  "manifest_id": String,              // Unique identifier for the manifest
  "manifest_type": String,            // Type classification (e.g., "image", "video")
  "content_format": String,           // Format: "JSON", "CBOR", or "Binary"
  "manifest_json": Object,            // JSON manifest data (if applicable)
  "manifest_cbor": String,            // Base64-encoded CBOR data (if applicable)
  "manifest_binary": String,          // Base64-encoded binary data (if applicable)
  "created_at": DateTime,             // Timestamp of creation
  "sequence_number": Number,          // Monotonically increasing sequence number
  "hash": String,                     // SHA384 hash of the raw manifest content
  "signature": String                 // Ed25519 signature of the hash
}
```

#### `merkle_tree_state` Collection
Stores the current state of the Merkle tree:

```javascript
{
  "leaves": Array<LogLeaf>,           // All leaves in the tree
  "tree_size": Number,                // Current number of leaves
  "root_hash": String,                // Current Merkle root hash
  "updated_at": DateTime              // Last update timestamp
}
```

**Note:** Root hash is recomputed from leaves during tree loading to ensure integrity, as the storage itself isn't tamper-proof.

### 2. Cryptographic Layer

#### Hashing (via atlas-common)
- **Algorithm**: SHA384 (48-byte output) - Default algorithm in atlas-common
- **Library**: `atlas-common::hash` module
- **Encoding**: Base64 for storage and transmission
- **Functions Used**: 
  - `calculate_hash()` - Default SHA384 hashing
  - `calculate_hash_with_algorithm()` - Specific algorithm support
  - `verify_hash()` - Constant-time verification
- **Usage**: 
  - Content hashing for manifests
  - Merkle tree node hashing
  - Leaf data includes all metadata to prevent tampering

#### Input Validation (via atlas-common)
- **Library**: `atlas-common::validation` module
- **Manifest ID Validation**: 
  - C2PA URN format: `urn:c2pa:UUID[:claim_generator[:version_reason]]`
  - Plain UUIDs: `123e4567-e89b-12d3-a456-426614174000`
  - Alphanumeric strings with hyphens, underscores, dots (max 256 chars)
- **Hash Format Validation**: Ensures proper hex encoding and length
- **Functions Used**:
  - `validate_manifest_id()` - Comprehensive ID validation
  - `validate_hash_format()` - Hash format verification
  - `ensure_c2pa_urn()` - URN format normalization

#### Digital Signatures
- **Algorithm**: Ed25519
- **Key Storage**: PKCS8 format, persisted to disk
- **Signing Process**: 
  1. Hash the manifest content with SHA384 (via atlas-common)
  2. Sign the hash with Ed25519 private key
  3. Store base64-encoded signature

#### Merkle Tree Structure
- **Leaf Format**: `"leaf:v0:{manifest_id}:{sequence_number}:{timestamp}:{content_hash}"`
- **Node Format**: `"node:{left_hash}:{right_hash}"`
- **Tree Construction**: Binary tree with RFC 6962-inspired structure
- **Odd Nodes**: Promoted to next level without pairing
- **Hashing**: Delegates to atlas-common for all hash operations

### 3. API Layer

#### Manifest Operations

**POST /manifests/{id}**
- Stores a new manifest
- Validates input size (max 10MB) and ID format using atlas-common
- Computes hash and signature using atlas-common
- Assigns sequence number
- Updates Merkle tree
- Returns manifest metadata

**GET /manifests/{id}**
- Retrieves manifest by ID
- Content negotiation based on Accept header
- Returns appropriate format (JSON/CBOR/Binary)

**GET /manifests**
- Lists manifests with pagination
- Query parameters: `limit`, `skip`, `manifest_type`, `format`
- Sorted by sequence number

#### Merkle Tree Operations

**GET /manifests/{id}/proof**
- Generates inclusion proof for a manifest
- Returns:
  ```json
  {
    "manifest_id": "string",
    "leaf_index": 0,
    "leaf_hash": "string",
    "merkle_path": ["hash1", "hash2"],
    "root_hash": "string",
    "tree_size": 0
  }
  ```

**POST /merkle/verify**
- Verifies an inclusion proof
- Validates proof against current tree state

**GET /merkle/consistency**
- Generates consistency proof between tree sizes
- Query parameters: `old_size`, `new_size`

**GET /merkle/root/{size}**
- Computes historical root for specific tree size
- Enables verification of past states

## Data Flow

### 1. Manifest Storage Flow

```
Client Request → Validation → Hash Computation → Signature Generation
     ↓                                                    ↓
Store in MongoDB ← Sequence Assignment ← Merkle Tree Update
     ↓
Return Response with Metadata
```

### 2. Proof Generation Flow

```
Proof Request → Find Leaf Position → Generate Merkle Path
     ↓                                        ↓
Compute Sibling Hashes ← Build Path from Leaf to Root
     ↓
Return Inclusion Proof
```

### 3. Verification Flow

```
Proof + Current Tree → Validate Tree Size → Verify Manifest ID
     ↓                                              ↓
Compute Leaf Hash → Traverse Merkle Path → Compare Root Hashes
     ↓
Return Verification Result
```

## Security Properties

### 1. Append-Only Guarantee
- Sequence numbers ensure ordering
- No deletion or modification operations
- Historical roots can be computed for any past size

### 2. Tamper Detection
- All leaf data included in hash computation via atlas-common
- Changing any field (manifest_id, sequence_number, timestamp) changes the hash
- Root hash changes if any leaf is modified

### 3. Cryptographic Integrity
- Ed25519 signatures prevent unauthorized modifications
- SHA384 (via atlas-common) provides collision resistance
- Constant-time hash comparison prevents timing attacks
- Merkle tree enables efficient verification

### 4. Verification Capabilities
- **Inclusion Proofs**: Prove a manifest exists in the log
- **Consistency Proofs**: Prove append-only property between states
- **Historical Verification**: Verify past states of the tree

## Implementation Details

### Module Structure

```
storage_service/
├── src/
│   ├── main.rs              # HTTP server and API endpoints
│   └── merkle_tree/         # Merkle tree implementation
│       ├── mod.rs           # Module exports
│       ├── hasher.rs        # Hashing trait (imported from atlas-common)
│       ├── proof.rs         # Proof structures and MerkleProof trait
│       └── tree.rs          # Core Merkle tree logic
```

### Dependencies

#### Core Dependencies
- **atlas-common**: Standardized cryptographic operations and validation
  - Features: `["hash", "validation"]`
  - Provides: SHA256/384/512 hashing, manifest validation, hash verification
- **actix-web**: HTTP server framework
- **mongodb**: Database driver
- **ring**: Ed25519 signatures
- **serde**: Serialization/deserialization

#### Atlas-Common Integration
- **Hashing**: All SHA operations delegated to `atlas-common::hash`
- **Validation**: Manifest ID and hash validation via `atlas-common::validation`
- **Consistency**: Ensures uniform cryptographic behavior across Atlas framework
- **Future-Proof**: Centralized updates to cryptographic functions

### Key Design Decisions

1. **SHA384 over SHA256**: Provides additional security margin (consistent with atlas-common default)
2. **atlas-common Integration**: Eliminates code duplication and ensures consistency
3. **In-Memory Merkle Tree**: Fast proof generation with MongoDB persistence
4. **Flexible Content Formats**: Supports JSON, CBOR, and binary manifests
5. **Synchronous Tree Updates**: Ensures consistency but may impact latency
6. **Unified Validation**: Uses atlas-common for all validation logic

### Performance Considerations

1. **Tree Reconstruction**: On startup, rebuilds from MongoDB if needed
2. **Proof Generation**: O(log n) time complexity
3. **Storage Growth**: Linear with number of manifests
4. **Concurrent Access**: Read-write lock on Merkle tree

## Configuration

Environment variables:
- `MONGODB_URI`: MongoDB connection string
- `DB_NAME`: Database name (default: "c2pa_manifests")
- `SERVER_HOST`: Server bind address (default: "0.0.0.0")
- `SERVER_PORT`: Server port (default: "8080")
- `KEY_PATH`: Ed25519 key file path

## Future Enhancements

1. **Batch Operations**: Support bulk manifest insertion
2. **Witness Cosigning**: Multiple signatures for enhanced trust
3. **Checkpointing**: Periodic signed checkpoints for faster sync
4. **Distributed Consensus**: Multi-node deployment with consensus
5. **Audit Logs**: Detailed operation logging for compliance

## References

- [RFC 6962](https://datatracker.ietf.org/doc/html/rfc6962): Certificate Transparency
- [C2PA Specification](https://c2pa.org/specifications/): Content Authenticity Initiative
- [Ed25519](https://ed25519.cr.yp.to/): High-speed signatures
- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree): Cryptographic hash trees
- [atlas-common](https://github.com/your-org/atlas-common): Shared cryptographic utilities