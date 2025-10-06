# C2PA Transparency Log Service

A cryptographically secure, append-only storage system for Content Authenticity Initiative (C2PA) manifests with verifiable transparency log capabilities.

⚠️ **Disclaimer**: This project is currently in active development. The code is **not stable** and **not intended for use in production environments**. Interfaces, features, and behaviors are subject to change without notice.

## Features

- **Verifiable Transparency Log**: Merkle tree-based proof system for manifest inclusion
- **Cryptographic Security**: Ed25519 signatures and SHA384 hashing via atlas-common
- **Multiple Content Formats**: Support for JSON, CBOR, and binary manifests
- **Append-Only Guarantee**: Immutable storage with sequence numbers
- **Proof Generation**: Inclusion and consistency proofs for third-party verification
- **RESTful API**: Easy integration with existing systems

## Documentation

- [Architecture Documentation](./ARCHITECTURE.md) - Detailed system design and implementation details
- [API Reference](#api-reference) - Complete API endpoint documentation

## Quick Start

### Prerequisites

- Rust 1.70+ 
- MongoDB 4.0+
- OpenSSL development libraries

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd storage_service
```
2. Build the project:
```bash
cargo build --release
```

3. Set up environment variables:
```bash
export MONGODB_URI="mongodb://localhost:27017"
export DB_NAME="c2pa_manifests"
export SERVER_HOST="0.0.0.0"
export SERVER_PORT="8080"
export KEY_PATH="transparency_log_key.pem"
```

4. Run the service:
```bash
cargo run --release
```

The service will start at `http://localhost:8080`.

## Usage Examples

### Store a Manifest

```bash
# JSON manifest
curl -X POST http://localhost:8080/manifests/my-manifest-123 \
  -H "Content-Type: application/json" \
  -d '{
    "manifest_type": "image",
    "data": "example content"
  }'

# CBOR manifest
curl -X POST http://localhost:8080/manifests/my-manifest-456 \
  -H "Content-Type: application/cbor" \
  --data-binary @manifest.cbor

# Binary manifest with type parameter
curl -X POST http://localhost:8080/manifests/my-manifest-789?manifest_type=video \
  -H "Content-Type: application/octet-stream" \
  --data-binary @manifest.bin

# C2PA URN format
curl -X POST http://localhost:8080/manifests/urn:c2pa:123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{"manifest_type": "model", "data": "ML model manifest"}'
```

### Get Inclusion Proof

```bash
curl http://localhost:8080/manifests/my-manifest-123/proof
```

Response:
```json
{
  "manifest_id": "my-manifest-123",
  "leaf_index": 42,
  "leaf_hash": "base64_hash...",
  "merkle_path": ["hash1", "hash2", "hash3"],
  "root_hash": "base64_root_hash...",
  "tree_size": 100
}
```

### Verify Inclusion Proof

```bash
curl -X POST http://localhost:8080/merkle/verify \
  -H "Content-Type: application/json" \
  -d '{
    "manifest_id": "my-manifest-123",
    "leaf_index": 42,
    "leaf_hash": "base64_hash...",
    "merkle_path": ["hash1", "hash2", "hash3"],
    "root_hash": "base64_root_hash...",
    "tree_size": 100
  }'
```

## API Reference

### Manifest Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/manifests/{id}` | Store a new manifest |
| GET | `/manifests/{id}` | Retrieve a manifest by ID |
| GET | `/manifests` | List manifests with pagination |
| GET | `/types/{type}/manifests` | List manifests by type |

### Merkle Tree Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/manifests/{id}/proof` | Get inclusion proof for a manifest |
| GET | `/merkle/root` | Get current Merkle root |
| POST | `/merkle/verify` | Verify an inclusion proof |
| GET | `/merkle/stats` | Get tree statistics |
| GET | `/merkle/consistency` | Get consistency proof between sizes |
| POST | `/merkle/consistency/verify` | Verify consistency proof |
| GET | `/merkle/root/{size}` | Get historical root for specific size |

### Query Parameters

#### List Manifests (`GET /manifests`)
- `limit` - Maximum number of results (default: 100)
- `skip` - Number of results to skip (default: 0)
- `manifest_type` - Filter by manifest type
- `format` - Filter by content format (json/cbor/binary)

#### Consistency Proof (`GET /merkle/consistency`)
- `old_size` - Old tree size
- `new_size` - New tree size

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_merkle_tree_multiple_leaves

# Test atlas-common integration
cargo test test_atlas_common_integration
```

### Project Structure

```
storage_service/
├── Cargo.toml               # Dependencies including atlas-common
├── README.md
├── ARCHITECTURE.md
└── src/
    ├── main.rs              # HTTP server and API endpoints
    ├── tests.rs             # Integration tests
    └── merkle_tree/         # Merkle tree implementation
        ├── mod.rs
        ├── hasher.rs        # Wrapper for atlas-common
        ├── proof.rs         # Proof structures and traits
        └── tree.rs          # Core tree implementation
```

### Key Dependencies

- **atlas-common**: Provides standardized hashing (SHA256/384/512) and validation utilities
- **actix-web**: Web framework for HTTP API
- **mongodb**: Database driver
- **ring**: Cryptographic primitives for Ed25519 signatures
- **serde**: Serialization/deserialization

## Security Considerations

1. **Private Key Protection**: The Ed25519 private key is stored in a file. Ensure proper file permissions and consider using a HSM in production.

2. **Input Validation**: Manifest IDs are validated using atlas-common's validation functions:
   - C2PA URN format: `urn:c2pa:UUID[:claim_generator[:version_reason]]`
   - Plain UUIDs: `123e4567-e89b-12d3-a456-426614174000`
   - Alphanumeric strings with hyphens, underscores, and dots (max 256 chars)

3. **Cryptographic Security**: 
   - SHA384 hashing (default) via atlas-common
   - Constant-time hash comparison to prevent timing attacks
   - Ed25519 signatures for content authenticity

4. **Size Limits**: Maximum manifest size is 10MB to prevent DoS attacks.

5. **Append-Only**: No deletion or modification operations are supported to maintain log integrity.

## Performance

- **Proof Generation**: O(log n) time complexity
- **Storage**: Linear growth with number of manifests
- **Verification**: Constant time for individual proofs
- **Tree Reconstruction**: O(n) on startup if needed
- **Hashing**: Optimized SHA384 implementation via atlas-common

## Troubleshooting

### MongoDB Connection Failed
```bash
# Check MongoDB is running
sudo systemctl status mongod

# Verify connection string
mongo mongodb://localhost:27017
```

### Key Generation Failed
```bash
# Check write permissions
ls -la transparency_log_key.pem

# Generate key manually
openssl genpkey -algorithm Ed25519 -out transparency_log_key.pem
```

### Hash Validation Errors
```bash
# Test hash validation
curl -X POST http://localhost:8080/manifests/test \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'

# Check logs for validation details
tail -f /var/log/transparency_log.log
```

### Large Manifest Rejection
- Default limit is 10MB
- Adjust `MAX_MANIFEST_SIZE` in `main.rs` if needed

## Acknowledgments

- [C2PA](https://c2pa.org/) - Content Authenticity Initiative
- [RFC 6962](https://datatracker.ietf.org/doc/html/rfc6962) - Certificate Transparency
- [MongoDB](https://www.mongodb.com/) - Database
- [Actix Web](https://actix.rs/) - Web framework